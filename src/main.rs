extern crate getopts;
extern crate quvyn;

use std::{env};
use std::process::exit;

use getopts::Options;

const DEFAULT_REPO_PATH: &str = "/var/lib/quvyn/repository";
const DEFAULT_APP_PATH: &str = "vue";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("a", "app", &format!("Specify path for the frontend app. Without this option the app is assumed in {}.", DEFAULT_APP_PATH), "PATH");
    opts.optopt("r", "repo", &format!("Specify path for the repository. Without this option the repository is stored in {}.", DEFAULT_REPO_PATH), "PATH");
    opts.optflag("", "reset", "Reset the repository. Or in other words, delete all comments. USE WITH EXTREME CAUTION!");
    opts.optflag("h", "help", "Display this help message");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            print!("{}", opts.usage(&f.to_string()));
            exit(1);
        }
    };
    if matches.opt_present("help") {
        print!("{}", opts.usage(&"Usage: quvyn [OPTIONS]"));
        return;
    }
    let app_path = matches.opt_get_default("app", String::from(DEFAULT_APP_PATH)).unwrap();
    let repo_path = matches.opt_get_default("repo", String::from(DEFAULT_REPO_PATH)).unwrap();
    let repo_reset = matches.opt_present("reset");

    quvyn::run(&app_path, &repo_path, repo_reset);
}

