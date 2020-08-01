use std::borrow::BorrowMut;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use glob::glob;
use gotham_derive::*;
use uuid::Uuid;

use crate::comment::Comment;
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
    }

    pub fn comment_with_id(&self, id: Uuid) -> Option<Comment> {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        list.iter().filter(|c| c.id == id).map(|c| c.clone()).last() // TODO: improve
    }

    pub fn comments_for_path(&self, path: &str) -> Vec<Comment> {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
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

    pub fn load_all_comments(&self) {
        for entry in glob(&format!("{}/*.json", self.path)).unwrap() {
            match entry {
                Ok(path) => self.load_comment(&path),
                Err(_) => {} // TODO
            }
        }
    }

    pub fn load_comment(&self, path: &Path) {
        println!("Loading comment from file: {}", path.display());
        let mut file = File::open(path).expect(&format!("Failed to open file {}", path.display()));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", path.display()));
        let comment = utils::from_json(&contents);
        self.add_comment(&comment);
    }

    pub fn save_comment(&self, comment: &Comment) {
        let filename = format!("{}/{}.json", self.path, comment.id.to_simple());
        println!("Saving comment to file: {}", filename);
        let mut file = File::create(&filename).expect(&format!("Failed to create file {}", &filename));
        let _result = file.write_all(utils::to_json(comment).as_ref());
        self.add_comment(comment); // TODO: there is no test to check that this happens after saving
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    impl CommentRepository {
        fn for_testing() -> CommentRepository {
            CommentRepository {
                path: "/r".to_owned(),
                comments: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[test]
    fn adding_comment_makes_it_available_in_list() {
        let repository = CommentRepository::for_testing();
        let comment = Comment::new("/test-topic/", "Test", None, None);

        repository.add_comment(&comment);
        let list = repository.all_comments();

        assert_eq!(1, list.len());
    }

    #[test]
    fn comment_can_be_retrieved_by_id() {
        let repository = CommentRepository::for_testing();
        let new_comment = Comment::new("/test-topic/", "Test", None, None);
        repository.add_comment(&new_comment);

        let result = repository.comment_with_id(new_comment.id);

        let returned_comment = result.expect("expected a comment");
        assert_eq!(returned_comment.id, new_comment.id);
        assert_eq!(returned_comment.content, "Test");
    }

    #[test]
    fn comments_can_be_retrieved_by_path() {
        let repository = CommentRepository::for_testing();
        repository.add_comment(&Comment::new("/test-topic/", "First comment", None, None));
        repository.add_comment(&Comment::new("/something-else/", "Second comment", None, None));

        let list = repository.comments_for_path("/test-topic/");

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].content, "First comment");
    }
}
