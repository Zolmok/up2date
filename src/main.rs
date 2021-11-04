use std::process::Command;
use std::fmt::{self, Formatter, Display};

struct Stdout(String);
struct App<'a> {
    command: &'a str,
    args: &'a[&'a str]
}

impl Display for Stdout {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn run(apps: &[&App]) {
    let mut cnt = 0;

    for app in apps.iter() {
        println!("{:?}", app.args);
        let output = Command::new(app.command)
            .args(app.args)
            .output();
        cnt = cnt + 1;

        match output {
            Err(error) => println!("{}", error),
            Ok(result) => println!("{}", {
                let results: String = match String::from_utf8(result.stdout) {
                    Err(error) => panic!("{}", error),
                    Ok(result) => result
                };

                Stdout(results)
            })
        }
    }
}

fn main() {
    let ls = App {
        command: "ls",
        args: &["-l", "-b"]
    };
    let apps: &[&App] = &[&ls];

    run(apps);
}
