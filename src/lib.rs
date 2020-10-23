#[macro_use]
extern crate gotham_derive;

use crate::repository::CommentRepository;

pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;

mod gotham_json;
mod gotham_cors;
mod gravatar;
mod markdown;


pub fn run(repo_path: String, repo_reset: bool, app_path: String, bind_addr: String, cors_origin: Option<String>)
{
    let repository = CommentRepository::new(&repo_path, repo_reset);
    repository.load_all_comments();
    webapi::run(repository, app_path, bind_addr, cors_origin);
}
