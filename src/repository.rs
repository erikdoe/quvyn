use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use glob::glob;
use gotham_derive::*;
use regex::Regex;

use crate::comment::Comment;
use crate::utils::calculate_hash;
use std::sync::{Mutex, Arc};
use std::borrow::BorrowMut;
use crate::utils;

#[derive(Clone, StateData)]
pub struct CommentRepository {
    path: String,
    comments: Arc<Mutex<Vec<Comment>>>,
}


impl CommentRepository {
    pub fn new(path: &str, reset: bool) -> Self {
        let repo = Self {
            path: path.to_owned(),
            comments: Arc::new(Mutex::new(Vec::new())),
        };
        if reset {
            repo.remove_storage_directory();
        }
        repo.create_storage_directory();
        repo
    }

    pub fn all_comments(&self) -> Vec<Comment> {
        let mut guard = self.comments.lock().unwrap();
        guard.borrow_mut().clone()
        // self.comments.iter().collect() // TODO: there must be a better way...
    }

    pub fn comments_for_path(&self, path: &str) -> Vec<Comment> {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut().clone();
        list.iter().filter(|c| c.path == path).map(|c| c.clone()).collect()
    }

    pub fn add_comment(&self, comment: &Comment) {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        list.push(comment.clone());
    }

    fn create_storage_directory(&self) {
        fs::create_dir_all(&self.path).expect(&format!("Failed to create directory at {}", &self.path));
    }

    fn remove_storage_directory(&self) {
        if !fs::metadata(&self.path).is_ok() {
            return;
        }
        fs::remove_dir_all(&self.path).expect(&format!("Failed to remove directory at {}", &self.path));
    }

    pub fn load_all_comments(&mut self) {
        for entry in glob(&format!("{}/**/*.json", self.path)).unwrap() {
            match entry {
                Ok(path) => self.load_comment(&path.display().to_string()),
                Err(_) => {} // TODO
            }
        }
    }

    pub fn load_comment(&mut self, path: &str) {
        println!("Loading comment from file: {}", path);
        let mut file = File::open(path).expect(&format!("Failed to open file {}", path));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", path));
        let comment = utils::from_json(&contents);
        self.add_comment(&comment);
    }

    pub fn save_comment(&self, comment: &Comment) {
        let path = self.path_for_comment(&comment);
        fs::create_dir_all(&path).expect(&format!("Failed to create directory at {}", &path));
        let filename = self.filename_for_comment(&comment);
        println!("Saving comment to file: {}", filename);
        let mut file = File::create(&filename).expect(&format!("Failed to create file {}", &filename));
        let _result = file.write_all(utils::to_json(comment).as_ref());
        self.add_comment(comment); // TODO: there is no test to check that this happens after saving
    }

    fn path_for_comment(&self, comment: &Comment) -> String {
        // we assume path is sanitised to include slashes and no query parameters and anchors
        let regex = Regex::new(r"[^0-9A-Za-z/-]").unwrap();
        let safe_path = regex.replace_all(&comment.path, "");
        format!("{}{}", self.path, safe_path)
    }

    fn filename_for_comment(&self, comment: &Comment) -> String {
        format!("{}{}.json", self.path_for_comment(comment), calculate_hash(&comment))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    impl CommentRepository {
        fn for_testing() -> CommentRepository {
            CommentRepository {
                path: "/r".to_owned(),
                comments: Arc::new(Mutex::new(Vec::new()))
            }
        }
    }

    #[test]
    fn adding_comment_makes_it_available_in_list() {
        let repository = CommentRepository::for_testing();
        let comment = Comment::new("/test-topic/", "Test");
        repository.add_comment(&comment);

        let list = repository.all_comments();

        assert_eq!(1, list.len());
    }

    #[test]
    fn path_specific_list_contains_comments_for_path() {
        let repository = CommentRepository::for_testing();
        repository.add_comment(&Comment::new("/test-topic/", "First comment"));
        repository.add_comment(&Comment::new("/something-else/", "Second comment"));

        let list = repository.comments_for_path("/test-topic/");

        assert_eq!(list.len(), 1);
    }

    #[test]
    fn storage_path_for_comment_concatenates_core_parts() {
        let repository = CommentRepository::for_testing();
        let comment = Comment::new("/test-topic/", "Test");

        let path = repository.path_for_comment(&comment);

        assert_eq!(path, "/r/test-topic/");
    }

    #[test]
    fn storage_path_for_comment_removes_non_ascii_chars() {
        let repository = CommentRepository::for_testing();
        let comment = Comment::new("/t&*e#öst/", "Test");

        let path = repository.path_for_comment(&comment);

        assert_eq!(path, "/r/test/");
    }

    #[test]
    fn filename_for_comment_is_comment_hash() {
        let repository = CommentRepository::for_testing();
        let comment = Comment::new("/test-topic/", "Test");

        let path = repository.filename_for_comment(&comment);

        assert_eq!(path, format!("/r/test-topic/{}.json", calculate_hash(&comment)));
    }
}
