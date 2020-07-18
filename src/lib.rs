use crate::repository::CommentRepository;

pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;

pub fn run(repo_path: &str)
{
    let repository = CommentRepository::new(repo_path, false);
    let address = format!("{}:{}", "localhost", 8080);
    webapi::run(repository, address);
}
