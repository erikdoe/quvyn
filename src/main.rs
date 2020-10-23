extern crate getopts;
extern crate quvyn;

use std::{env};
use std::process::exit;

use getopts::Options;

const DEFAULT_BIND_ADDR: &str = "localhost:8080";
const DEFAULT_REPO_PATH: &str = "/var/lib/quvyn/repository";
const DEFAULT_APP_PATH: &str = "vue";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("r", "repo", &format!("Specify path for the repository. By default the repository is stored in {}.", DEFAULT_REPO_PATH), "PATH");
    opts.optflag("", "reset", "Reset the repository. Or in other words, delete all comments. USE WITH EXTREME CAUTION!");
    opts.optopt("a", "app", &format!("Specify path for the frontend app. By default the app is assumed in {}.", DEFAULT_APP_PATH), "PATH");
    opts.optopt("b", "bind", &format!("Specify address and port for the server. By default the server binds to {}. ", DEFAULT_BIND_ADDR), "HOST:PORT");
    opts.optopt("o", "origin", &format!("Specify an origin allowed for CORS. By default no origins are allowed."), "URL");
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

    let repo_path = matches.opt_get_default("repo", String::from(DEFAULT_REPO_PATH)).unwrap();
    let repo_reset = matches.opt_present("reset");
    let app_path = matches.opt_get_default("app", String::from(DEFAULT_APP_PATH)).unwrap();
    let bind_addr = matches.opt_get_default("bind", String::from(DEFAULT_BIND_ADDR)).unwrap();
    let cors_origin = matches.opt_str("origin");

    quvyn::run(repo_path, repo_reset, app_path, bind_addr, cors_origin);
}

