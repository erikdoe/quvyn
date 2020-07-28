extern crate getopts;
extern crate quvyn;

use std::env;

use getopts::Options;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "path", "Specify path for the repository. Without this option the repository is stored in /var/lib/quvyn/repository.", "PATH");
    opts.optflag("h", "help", "Display this help message");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let path = matches.opt_get_default("p", String::from("/var/lib/quvyn/repository")).unwrap();

    quvyn::run(&path);
}


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] files", program);
    print!("{}", opts.usage(&brief));
}
