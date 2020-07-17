pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;

pub fn run(_files: Vec<String>)
{
    let addr = format!("{}:{}", "localhost", 8080);
    webapi::run(addr);
}
