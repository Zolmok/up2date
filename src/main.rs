use std::env::consts::OS;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::{Command, ExitStatus, Output};

use sys_info::*;

struct Args(Vec<String>);

struct App {
    command: String,
    args: Vec<String>,
}

struct CargoApps {
    to_update: Vec<String>,
    skipped: Vec<String>,
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

/// Run an app and display its output
///
/// # Arguments
///
/// * `app` - An app of type `App`
///
/// # Errors
///
/// Returns an error if the command fails to spawn or wait
fn run_status(app: &App) -> Result<ExitStatus, std::io::Error> {
    Command::new(&app.command)
        .args(&app.args)
        .spawn()?
        .wait()
}

/// Run an app and return its output
///
/// # Arguments
///
/// * `app` - An app of type `App`
///
/// # Errors
///
/// Returns an error if the command fails to execute
fn run_output(app: &App) -> Result<Output, std::io::Error> {
    Command::new(&app.command).args(&app.args).output()
}

/// Run a list of apps and print out the command and its arguments before running
///
/// # Arguments
///
/// * `apps` - A vector of apps to run
///
/// # Errors
///
/// Returns an error if any command fails to execute
fn run_apps(apps: &[App]) -> Result<(), Box<dyn Error>> {
    for app in apps.iter() {
        println!();
        println!("========================");
        println!("$ {} {}", app.command, Args(app.args.to_owned()));
        println!("========================");

        run_status(app)?;
    }
    Ok(())
}

/// Run an app optionally - if it fails to execute, print a warning and continue
///
/// This function is useful for commands that may not be installed on all systems.
/// If the command is not found or fails to spawn, a warning is printed and
/// execution continues rather than returning an error.
///
/// # Arguments
///
/// * `app` - An app of type `App`
/// * `description` - A description of what the app does (for warning message)
fn run_optional(app: &App, description: &str) {
    println!();
    println!("========================");
    println!("$ {} {}", app.command, Args(app.args.to_owned()));
    println!("========================");

    match Command::new(&app.command).args(&app.args).spawn() {
        Ok(mut child) => {
            match child.wait() {
                Ok(_status) => {}
                Err(error) => {
                    println!(
                        "Warning: {} failed to complete: {}",
                        description, error
                    );
                }
            }
        }
        Err(error) => {
            match error.kind() {
                std::io::ErrorKind::NotFound => {
                    println!(
                        "Warning: {} not found, skipping {}",
                        app.command, description
                    );
                }
                _ => {
                    println!(
                        "Warning: failed to run {}: {}",
                        app.command, error
                    );
                }
            }
        }
    }
}

/// Parse orphan package names from command output
///
/// # Arguments
///
/// * `stdout` - Raw bytes from command stdout
///
/// # Returns
///
/// A vector of package names with empty lines filtered out
fn parse_orphan_packages(stdout: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Run an app, check its output, conditionally run a second app
///
/// Should be passed an array with exactly 2 Apps.
/// The first App is run and its output is checked.
/// If there is output, that is appended to the second
/// Apps argument list and that App is run
///
/// # Arguments
///
/// * `apps` - A vector of exactly 2 Apps
///
/// # Errors
///
/// Returns an error if the slice doesn't contain exactly 2 apps or if command execution fails
///
/// # Examples
/// ```
/// let first_app = App {
///     command: String::from("some-command"),
///     args: vec![String::from("some-argument")]
/// };
/// let second_app = App {
///     command: String::from("some-command"),
///     args: vec![String::from("some-argument")]
/// };
///
/// let apps_with_response: &[App] = &[first_app, second_app];
/// run_with_response(apps_with_response);
/// ```
fn run_with_response(apps: &[App]) -> Result<(), Box<dyn Error>> {
    if apps.len() != 2 {
        return Err(format!(
            "run_with_response requires exactly 2 apps, got {}",
            apps.len()
        )
        .into());
    }

    let first = &apps[0];
    let second = &apps[1];

    let result = run_output(first)?;

    let args = parse_orphan_packages(&result.stdout);

    if !args.is_empty() {
        let second_with_orphans = App {
            command: second.command.clone(),
            args: [&second.args[..], &args[..]].concat(),
        };

        run_apps(&[second_with_orphans])?;
    }

    Ok(())
}

/// Parse cargo install --list output to extract app names
///
/// Separates packages into those to update (from crates.io) and those to skip
/// (locally installed via `cargo install --path`).
///
/// # Arguments
///
/// * `output` - The string output from `cargo install --list`
///
/// # Returns
///
/// A CargoApps struct containing:
/// - `to_update`: packages from crates.io that should be updated
/// - `skipped`: locally installed packages (identified by path in parentheses)
fn parse_cargo_apps(output: &str) -> CargoApps {
    let mut to_update = Vec::new();
    let mut skipped = Vec::new();

    for line in output.lines() {
        // Skip indented lines (these are binary names, not package names)
        if line.starts_with(' ') {
            continue;
        }

        // Extract the package name (first word before space)
        match line.split(' ').next() {
            Some(app) => {
                if app.is_empty() {
                    continue;
                }

                // Check if this is a local install (has path in parentheses)
                // Local: "dev v1.2.0 (/home/user/path/to/dev):"
                // Crates.io: "bat v0.26.1:"
                if line.contains("(/") {
                    skipped.push(app.to_string());
                } else {
                    to_update.push(app.to_string());
                }
            }
            None => {
                continue;
            }
        }
    }

    CargoApps { to_update, skipped }
}

/// Parse the output of `cargo install --list` and build a command to update the apps from the list
///
/// Locally installed packages (via `cargo install --path`) are skipped with a notice.
/// Packages from crates.io are updated.
///
/// # Arguments
///
/// * `app` - An app of type `App`
///
/// # Errors
///
/// Returns an error if the cargo command fails or if the output cannot be parsed
fn run_with_cargo(app: App) -> Result<(), Box<dyn Error>> {
    let output = run_output(&app)?;
    let result = std::str::from_utf8(&output.stdout)?;

    let cargo_apps = parse_cargo_apps(result);

    // Print skip notices for local installs
    for skipped_app in cargo_apps.skipped.iter() {
        println!("Skipping {} (local install)", skipped_app);
    }

    // Update packages from crates.io
    for cargo_app in cargo_apps.to_update.iter() {
        let cargo_install_app = App {
            command: String::from("cargo"),
            args: vec!["install".to_string(), cargo_app.clone()],
        };

        run_apps(&[cargo_install_app])?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if OS == "linux" {
        let release = linux_os_release()?.id;

        match release.as_deref() {
            Some("ubuntu") | Some("pop") => {
                let apt_update = App {
                    command: String::from("sudo"),
                    args: vec!["apt-get".to_string(), "update".to_string()],
                };
                let apt_upgrade = App {
                    command: String::from("sudo"),
                    args: vec![
                        "apt-get".to_string(),
                        "upgrade".to_string(),
                        "-y".to_string(),
                        "--allow-downgrades".to_string(),
                        "--with-new-pkgs".to_string(),
                    ],
                };
                let apt_remove = App {
                    command: String::from("sudo"),
                    args: vec![
                        "apt-get".to_string(),
                        "autoremove".to_string(),
                        "-y".to_string(),
                    ],
                };
                let apps: &[App] = &[apt_update, apt_upgrade, apt_remove];

                run_apps(apps)?;
            }
            Some("arch") | Some("endeavouros") => {
                let pacman_keyring = App {
                    command: String::from("sudo"),
                    args: vec![
                        "pacman".to_string(),
                        "--noconfirm".to_string(),
                        "-S".to_string(),
                        "archlinux-keyring".to_string(),
                    ],
                };
                let pacman_update = App {
                    command: String::from("sudo"),
                    args: vec![
                        "pacman".to_string(),
                        "--noconfirm".to_string(),
                        "-Syu".to_string(),
                    ],
                };
                let pacman_orphan_check = App {
                    command: String::from("pacman"),
                    args: vec!["-Qtdq".to_string()],
                };
                let pacman_orphan_remove = App {
                    command: String::from("sudo"),
                    args: vec![
                        "pacman".to_string(),
                        "--noconfirm".to_string(),
                        "-Rns".to_string(),
                    ],
                };

                let yay_update = App {
                    command: String::from("yay"),
                    args: vec!["--noconfirm".to_string(), "-Syu".to_string()],
                };
                let yay_orphan_check = App {
                    command: String::from("yay"),
                    args: vec!["-Qtdq".to_string()],
                };
                let yay_orphan_remove = App {
                    command: String::from("yay"),
                    args: vec!["--noconfirm".to_string(), "-Rns".to_string()],
                };

                let apps: &[App] = &[pacman_keyring, pacman_update, yay_update];
                run_apps(apps)?;

                // Remove pacman orphan packages
                run_with_response(&[pacman_orphan_check, pacman_orphan_remove])?;

                // Remove yay orphan packages
                run_with_response(&[yay_orphan_check, yay_orphan_remove])?;
            }
            Some(os_name) => {
                return Err(format!("ERROR: not sure what OS this is: {}", os_name).into())
            }
            None => return Err("ERROR: not sure what OS this is".into()),
        }
    }

    if OS == "macos" {
        let brew_update = App {
            command: String::from("brew"),
            args: vec!["update".to_string()],
        };
        let brew_upgrade = App {
            command: String::from("brew"),
            args: vec!["upgrade".to_string()],
        };
        let brew_cleanup = App {
            command: String::from("brew"),
            args: vec!["cleanup".to_string()],
        };
        let apps: &[App] = &[brew_update, brew_upgrade, brew_cleanup];

        run_apps(apps)?;
    }

    // update rust, should be the same on all platforms
    // uses run_optional because rustup may not be installed (e.g., omarchy uses system rust)
    let rust_update = App {
        command: String::from("rustup"),
        args: vec!["update".to_string()],
    };
    run_optional(&rust_update, "Rust toolchain update");

    // update vim
    let neovim_update = App {
        command: String::from("nvim"),
        args: vec![
            "--headless".to_string(),
            "+Lazy! sync".to_string(),
            "+qa".to_string(),
        ],
    };
    run_apps(&[neovim_update])?;

    // update all rust apps installed with cargo
    let cargo_list_apps = App {
        command: String::from("cargo"),
        args: vec!["install".to_string(), "--list".to_string()],
    };

    run_with_cargo(cargo_list_apps)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_apps_basic() {
        let input = "ripgrep v14.1.0:\n    rg\nbat v0.24.0:\n    bat\n";
        let result = parse_cargo_apps(input);
        assert_eq!(result.to_update, vec!["ripgrep", "bat"]);
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_parse_cargo_apps_filters_indented_lines() {
        let input = "myapp v1.0.0:\n    myapp\n    myapp-cli\nanotherapp v2.0.0:\n    another\n";
        let result = parse_cargo_apps(input);
        assert_eq!(result.to_update, vec!["myapp", "anotherapp"]);
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_parse_cargo_apps_skips_local_installs() {
        // Local installs have path in parentheses, crates.io packages don't
        let input = "dev v1.2.0 (/home/user/src/dev):\n    dev\nbat v0.24.0:\n    bat\ntm v0.5.0 (/home/user/src/tm):\n    tm\n";
        let result = parse_cargo_apps(input);
        assert_eq!(result.to_update, vec!["bat"]);
        assert_eq!(result.skipped, vec!["dev", "tm"]);
    }

    #[test]
    fn test_parse_cargo_apps_empty_input() {
        let result = parse_cargo_apps("");
        assert!(result.to_update.is_empty());
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_parse_orphan_packages_basic() {
        let input = b"package1\npackage2\npackage3\n";
        let result = parse_orphan_packages(input);
        assert_eq!(result, vec!["package1", "package2", "package3"]);
    }

    #[test]
    fn test_parse_orphan_packages_filters_empty_lines() {
        let input = b"package1\n\npackage2\n\n\npackage3\n";
        let result = parse_orphan_packages(input);
        assert_eq!(result, vec!["package1", "package2", "package3"]);
    }

    #[test]
    fn test_parse_orphan_packages_empty_input() {
        let result = parse_orphan_packages(b"");
        assert!(result.is_empty());
    }

    #[test]
    fn test_args_display() {
        let args = Args(vec!["apt-get".to_string(), "update".to_string()]);
        assert_eq!(format!("{}", args), "apt-get update");
    }

    #[test]
    fn test_args_display_empty() {
        let args = Args(vec![]);
        assert_eq!(format!("{}", args), "");
    }

    #[test]
    fn test_run_optional_handles_missing_command() {
        // Should not panic when command doesn't exist
        let app = App {
            command: String::from("nonexistent_command_that_does_not_exist_12345"),
            args: vec![],
        };
        run_optional(&app, "test operation");
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_run_optional_handles_valid_command() {
        // Should not panic when command exists and succeeds
        let app = App {
            command: String::from("true"),
            args: vec![],
        };
        run_optional(&app, "test operation");
        // If we get here without panicking, the test passes
    }
}
