use std::env::consts::OS;
use sys_info::*;

use scuttle::{App, Args};

extern crate scuttle;
extern crate sys_info;

/// Run a list of apps and print out the command and it's arguments before running
///
/// # Arguments
///
/// * `apps` - A vector of apps to run
fn run_apps(apps: &[App]) {
    for app in apps.iter() {
        println!("");
        println!("========================");
        println!("$ {} {}", app.command, Args(app.args.to_owned()));
        println!("========================");

        match scuttle::run_status(app) {
            Err(error) => panic!("panic{}", error),
            Ok(_status) => continue,
        };
    }
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
/// # Examples
/// ```
/// let first_app = App {
///     command: String::from("some-command"),
///     args: vec![String::from("some-argument")]
/// };
/// let second_app = App {
///     command: String::from("some-command"),
///     args: vec!&[String::from("some-argument")]
/// };
///
/// let apps_with_response: &[App] = &[first_app, second_app];
/// run_with_response(apps_with_response);
/// ```
fn run_with_response(apps: &[App]) {
    let first = &apps[0];
    let second = &apps[1];

    match scuttle::run_output(&first) {
        Ok(result) => {
            if result.stdout.len() > 0 {
                let orphans = String::from_utf8_lossy(&result.stdout);
                let mut args: Vec<String> = orphans.split('\n').map(String::from).collect();

                // sometimes the last entry is empty so find and remove it
                for i in (0..args.len()).rev() {
                    if args[i] == "" {
                        args.swap_remove(i);
                    }
                }

                let second_with_orphans = App {
                    command: second.command.clone(),
                    args: [&second.args[..], &args[..]].concat(),
                };

                run_apps(&[second_with_orphans]);
            }
        }
        Err(error) => panic!("{}", error),
    }
}

/// Parse the output of `cargo install --list` and build a command to update the apps from the list
///
/// # Arguments
///
/// * `app` - An app of type `App`
fn run_with_cargo(app: App) {
    match scuttle::run_output(&app) {
        Ok(output) => match std::str::from_utf8(&output.stdout) {
            Ok(result) => {
                result.lines().for_each(move |line| {
                    if !line.starts_with(' ') {
                        let parts: Vec<&str> = line.split(' ').collect();
                        let cargo_app = parts[0];
                        let cargo_install_app = App {
                            command: String::from("cargo"),
                            args: vec!["install".to_string(), cargo_app.to_string()],
                        };

                        run_apps(&[cargo_install_app]);
                    }
                });
            }
            Err(error) => println!("error:{}", error),
        },
        Err(error) => panic!("panic:{}", error),
    };
}

fn main() {
    if OS == "linux" {
        let release = match linux_os_release() {
            Ok(value) => value.id,
            Err(error) => panic!("Error {}", error),
        };

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

                run_apps(apps);
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

                let yum_update = App {
                    command: String::from("yum"),
                    args: vec!["--noconfirm".to_string(), "-Syu".to_string()],
                };
                let yum_orphan_check = App {
                    command: String::from("yum"),
                    args: vec!["-Qtdq".to_string()],
                };
                let yum_orphan_remove = App {
                    command: String::from("yum"),
                    args: vec!["--noconfirm".to_string(), "-Rns".to_string()],
                };
                let apps: &[App] = &[pacman_keyring, pacman_update, yum_update];
                let apps_with_response: &[App] = &[
                    pacman_orphan_check,
                    pacman_orphan_remove,
                    yum_orphan_check,
                    yum_orphan_remove,
                ];

                run_apps(apps);
                run_with_response(apps_with_response);
            }
            Some(os_name) => panic!("ERROR: not sure what OS this is:{}", os_name),
            None => panic!("ERROR: not sure what OS this is"),
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

        run_apps(apps);
    }

    // update rust, should be the same on all platforms
    let rust_update = App {
        command: String::from("rustup"),
        args: vec!["update".to_string()],
    };
    // update vim
    let neovim_update = App {
        command: String::from("nvim"),
        args: vec![
            "--headless".to_string(),
            "-c".to_string(),
            "autocmd User PackerComplete quitall".to_string(),
            "-c".to_string(),
            "PackerUpdate".to_string(),
        ],
    };
    let apps: &[App] = &[rust_update, neovim_update];

    run_apps(apps);

    // update all rust apps installed with cargo
    let cargo_list_apps = App {
        command: String::from("cargo"),
        args: vec!["install".to_string(), "--list".to_string()],
    };

    run_with_cargo(cargo_list_apps);
}
