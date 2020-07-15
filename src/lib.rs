extern crate core;
extern crate gotham;
extern crate serde;
extern crate serde_json;

pub mod webapi;

pub fn run(_files: Vec<String>)
{
    let addr = format!("{}:{}", "localhost", 8080);
    webapi::run(addr);
}
