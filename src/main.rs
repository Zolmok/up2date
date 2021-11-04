use std::env::consts::OS;
use std::fmt::{self, Formatter, Display};
use std::process::Command;

struct Args(Vec<String>);
struct App {
    command: String,
    args: Vec<String>
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

fn run(apps: &[App]) {
    for app in apps.iter() {
        println!("");
        println!("========================");
        println!("$ {} {}", app.command, Args(app.args.clone()));
        println!("========================");

        let child = Command::new(app.command.clone())
            .args(app.args.clone())
            .spawn();

        match child {
            Err(error) => panic!("{}", error),
            Ok(mut result) => {
                match result.try_wait() {
                    Err(error) => panic!("{}", error),
                    Ok(Some(_status)) => continue,
                    Ok(None) => {
                        let wait = result.wait();

                        match wait {
                            Err(error) => panic!("{}", error),
                            Ok(_status) => continue
                        }
                    }
                };
            }
        };
    }
}

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
fn run_with_response(apps: &[App]) {
    let first = &apps[0];
    let second = &apps[1];

    println!("");
    println!("========================");
    println!("$ {} {}", first.command, Args(first.args.clone()));
    println!("========================");

    let first_child = Command::new(first.command.clone())
        .args(first.args.clone())
        .output();

    match first_child {
        Err(error) => panic!("{}", error),
        Ok(result) => {
            if result.stdout.len() > 0 {
                let orphans = String::from_utf8_lossy(&result.stdout);
                let args = format!("{} {}", Args(second.args.clone()), orphans);
                let second_with_orphans = App {
                    command: second.command.clone(),
                    args: vec![String::from(args)]
                };

                run(&[second_with_orphans]);
            }
        }
    }
}

fn main() {
    if OS == "linux" {
        let pacman_keyring = App {
            command: String::from("sudo"),
            args: vec![String::from("pacman"), String::from("--noconfirm"), String::from("-S"), String::from("archlinux-keyring")]
        };
        let pacman_update = App {
            command: String::from("sudo"),
            args: vec![String::from("pacman"), String::from("--noconfirm"), String::from("-Syu")]
        };
        let pacman_orphan_check = App {
            command: String::from("pacman"),
            args: vec![String::from("-Qtdq")]
        };
        let pacman_orphan_remove = App {
            command: String::from("sudo"),
            args: vec![String::from("pacman"), String::from("--noconfirm"), String::from("-Rns")]
        };
        let yay_update = App {
            command: String::from("yay"),
            args: vec![String::from("-Syu")]
        };
        let yay_cleanup = App {
            command: String::from("yay"),
            args: vec![String::from("-Yc")]
        };
        let apps: &[App] = &[pacman_keyring, pacman_update, yay_update, yay_cleanup];
        let apps_with_response: &[App] = &[pacman_orphan_check, pacman_orphan_remove];

        run(apps);
        run_with_response(apps_with_response);
    }

    if OS == "macos" {
        let brew_update = App {
            command: String::from("brew"),
            args: vec![String::from("update")]
        };
        let brew_upgrade = App {
            command: String::from("brew"),
            args: vec![String::from("upgrade")]
        };
        let brew_cask_upgrade = App {
            command: String::from("brew"),
            args: vec![String::from("cask"), String::from("upgrade")]
        };
        let brew_cleanup = App {
            command: String::from("brew"),
            args: vec![String::from("cleanup")]
        };
        let apps: &[App] = &[brew_update, brew_upgrade, brew_cask_upgrade, brew_cleanup];

        run(apps);
    }
}
