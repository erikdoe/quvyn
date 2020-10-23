use crate::repository::CommentRepository;

pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;

mod gotham_json;
mod gravatar;
mod markdown;


pub fn run(bind_addr: &str, app_path: &str, repo_path: &str, repo_reset: bool)
{
    let repository = CommentRepository::new(repo_path, repo_reset);
    repository.load_all_comments();
    webapi::run(app_path, repository, bind_addr);
}
