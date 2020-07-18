use crate::repository::CommentRepository;

pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;

pub fn run(repo_path: &str)
{
    let repo = CommentRepository::new(repo_path, false);
    let addr = format!("{}:{}", "localhost", 8080);
    webapi::run(repo, addr);
}
