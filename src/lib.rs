#[macro_use]
extern crate gotham_derive;

use std::process;

use crate::repository::CommentRepository;
use crate::notifier::Notifier;

pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;
pub mod importer;

mod gotham_json;
mod gotham_cors;
mod gravatar;
mod markdown;
mod notifier;
mod sendmail;


pub fn run(repo_path: String, repo_reset: bool, app_path: String, bind_addr: String,
           cors_origin: Option<String>, notify_addr: Option<String>)
{
    let mut repository = CommentRepository::new(&repo_path, repo_reset);
    if let Some(addr) = notify_addr {
        repository.set_notifier(Notifier::new(&addr))
    }
    repository.load_all_comments();
    webapi::run(repository, &app_path, &bind_addr, &cors_origin);
}


pub fn import(repo_path: String, repo_reset: bool, filename: String)
{
    let repository = CommentRepository::new(&repo_path, repo_reset);
    let result = importer::run(&filename, repository);
    if let Err(message) = result {
        println!("Error during import: {}", message);
        process::exit(1);
    }
}
