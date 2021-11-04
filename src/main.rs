use std::env::consts::OS;
use std::process::Command;

extern crate scuttle;
extern crate sys_info;

use sys_info::*;

/// Run an app, check its output, conditionally run a second
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
fn run_with_response(apps: &[scuttle::App]) {
    let first = &apps[0];
    let second = &apps[1];
    let first_child = Command::new(first.command.clone())
        .args(first.args.clone())
        .output();

    match first_child {
        Err(error) => panic!("{}", error),
        Ok(result) => {
            if result.stdout.len() > 0 {
                let orphans = String::from_utf8_lossy(&result.stdout);
                let mut args: Vec<&str> = orphans.split('\n').collect();

                // sometimes the last entry is empty so find and remove it
                for i in (0..args.len()).rev() {
                    if args[i] == "" {
                        args.swap_remove(i);
                    }
                }

                let second_with_orphans = scuttle::App {
                    command: second.command.clone(),
                    args: [&second.args[..], &args[..]].concat(),
                };

                scuttle::run_apps(&[second_with_orphans]);
            }
        }
    }
}

fn main() {
    if OS == "linux" {
        let release = match linux_os_release() {
            Ok(value) => value.id,
            Err(error) => panic!("Error {}", error)
        };

        match release.as_deref() {
            Some("pop") | Some("ubuntu") => {
                let apt_update = scuttle::App {
                    command: String::from("sudo"),
                    args: vec!["apt-get", "update"]
                };
                let apt_upgrade = scuttle::App {
                    command: String::from("sudo"),
                    args: vec!["apt-get", "upgrade", "-y", "--allow-downgrades", "--with-new-pkgs"]
                };
                let apt_remove = scuttle::App {
                    command: String::from("sudo"),
                    args: vec!["apt-get", "autoremove", "-y"]
                };
                let apps: &[scuttle::App] = &[apt_update, apt_upgrade, apt_remove];

                scuttle::run_apps(apps);
            },
            Some("arch") => {
                let pacman_keyring = scuttle::App {
                    command: String::from("sudo"),
                    args: vec!["pacman", "--noconfirm", "-S", "archlinux-keyring"]
                };
                let pacman_update = scuttle::App {
                    command: String::from("sudo"),
                    args: vec!["pacman", "--noconfirm", "-Syu"]
                };
                let pacman_orphan_check = scuttle::App {
                    command: String::from("pacman"),
                    args: vec!["-Qtdq"]
                };
                let pacman_orphan_remove = scuttle::App {
                    command: String::from("sudo"),
                    args: vec!["pacman", "--noconfirm", "-Rns"]
                };
                let apps: &[scuttle::App] = &[pacman_keyring, pacman_update];
                let apps_with_response: &[scuttle::App] = &[pacman_orphan_check, pacman_orphan_remove];

                scuttle::run_apps(apps);
                run_with_response(apps_with_response);
            },
            Some(&_) => panic!("ERROR: not sure what distribution this is"),
            None => panic!("ERROR: not sure what distribution this is")
        }
    }

    if OS == "macos" {
        let brew_update = scuttle::App {
            command: String::from("brew"),
            args: vec!["update"],
        };
        let brew_upgrade = scuttle::App {
            command: String::from("brew"),
            args: vec!["upgrade"],
        };
        let brew_cleanup = scuttle::App {
            command: String::from("brew"),
            args: vec!["cleanup"],
        };
        let apps: &[scuttle::App] = &[brew_update, brew_upgrade, brew_cleanup];

        scuttle::run_apps(apps);
    }

    // update rust, should be the same on all platforms
    let rust_update = scuttle::App {
        command: String::from("rustup"),
        args: vec!["update"],
    };
    let apps: &[scuttle::App] = &[rust_update];

    scuttle::run_apps(apps);
}
