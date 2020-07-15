extern crate quvyn;
extern crate getopts;

use getopts::Options;
use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
//    opts.optopt("o", "", "set output file name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    quvyn::run(matches.free);
}


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] files", program);
    print!("{}", opts.usage(&brief));
}
