use std::fs;
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use glob::glob;
use gotham_derive::*;
use uuid::Uuid;

use crate::comment::Comment;
use crate::notifier::Notifier;
use crate::utils;

#[derive(Clone, StateData)]
pub struct CommentRepository {
    path: String,
    comments: Arc<Mutex<Vec<Comment>>>,
    notifier: Option<Notifier>,
    should_reload: Arc<AtomicBool>,
}


impl CommentRepository {
    pub fn new(path: &str, reset: bool) -> Self {
        let repo = Self {
            path: path.to_owned(),
            comments: Arc::new(Mutex::new(Vec::new())),
            notifier: None,
            should_reload: Arc::new(AtomicBool::new(false)),
        };
        if reset {
            repo.remove_storage_directory();
        }
        repo.create_storage_directory();
        repo
    }

    pub fn set_reload_flag(&mut self, flag: &Arc<AtomicBool>) {
        self.should_reload = Arc::clone(&flag);
    }

    pub fn set_notifier(&mut self, notifier: Notifier) {
        self.notifier = Some(notifier)
    }

    pub fn all_comments(&self) -> Vec<Comment> {
        self.reload_all_comments();
        let mut guard = self.comments.lock().unwrap();
        guard.borrow_mut().clone()
    }

    pub fn comment_with_id(&self, id: Uuid) -> Option<Comment> {
        self.reload_all_comments();
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        list.iter().filter(|c| c.id == id).map(|c| c.clone()).last() // TODO: improve
    }

    pub fn comments_for_path(&self, path: &str) -> Vec<Comment> {
        self.reload_all_comments();
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        let mut list: Vec<Comment> = list.iter().filter(|c| c.path == path).map(|c| c.clone()).collect();
        list.sort_unstable_by_key(|c| c.timestamp);
        list
    }

    pub fn add_comment(&self, comment: &Comment) {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        list.push(comment.clone());
    }

    pub fn remove_comment(&self, comment: &Comment) -> bool {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        list.iter().position(|c| c.id == comment.id).map(|c| list.remove(c)).is_some()
    }

    fn remove_all_comments(&self) {
        let mut guard = self.comments.lock().unwrap();
        let list = guard.borrow_mut();
        list.clear();
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

    fn reload_all_comments(&self) {
        // TODO: this implementation is not entirely correct; another thread could see no comments
        if self.should_reload.swap(false, Ordering::Relaxed) {
            self.remove_all_comments();
            self.load_all_comments();
        }
    }

    pub fn load_all_comments(&self) {
        for entry in glob(&format!("{}/*.json", self.path)).unwrap() {
            match entry {
                Ok(path) => self.load_comment(&path),
                Err(_) => {} // TODO
            }
        }
    }

    fn load_comment(&self, path: &Path) {
        println!("Loading comment from file: {}", path.display());
        let mut file = File::open(path).expect(&format!("Failed to open file {}", path.display()));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", path.display()));
        let comment = utils::from_json(&contents);
        self.add_comment(&comment);
    }

    pub fn save_comment(&self, comment: &Comment) {
        let filename = format!("{}/{}.json", self.path, comment.id.as_simple());
        println!("Saving comment to file: {}", filename);
        let mut file = File::create(&filename).expect(&format!("Failed to create file {}", &filename));
        let _result = file.write_all(utils::to_json(comment).as_ref());
        self.add_comment(comment); // TODO: there is no test to check that this happens after saving
        if let Some(notifier) = &self.notifier {
            notifier.notify(comment)
        }
    }

    pub fn delete_comment(&self, comment: &Comment) {
        let filename = format!("{}/{}.json", self.path, comment.id.as_simple());
        println!("Deleting comment in file: {}", filename);
        std::fs::remove_file(&filename).expect(&format!("Failed to delete comment in file {}", &filename));
        self.remove_comment(comment);
    }
}


#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    impl CommentRepository {
        fn for_testing() -> CommentRepository {
            CommentRepository {
                path: "/r".to_owned(),
                comments: Arc::new(Mutex::new(Vec::new())),
                notifier: None,
                should_reload: Arc::new(AtomicBool::new(false)),
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
    fn removing_comment_makes_it_unavailable_in_list() {
        let repository = CommentRepository::for_testing();
        let comment1 = Comment::new("/test-topic/", "First comment", None, None);
        repository.add_comment(&comment1);
        repository.add_comment(&Comment::new("/test-topic/", "Second comment", None, None));

        repository.remove_comment(&comment1);

        assert_eq!(1, repository.all_comments().len());
        let found = repository.comment_with_id(comment1.id).is_some();
        assert_eq!(false, found);
    }

    #[test]
    fn comment_can_be_retrieved_by_id() {
        let repository = CommentRepository::for_testing();
        let new_comment = Comment::new("/test-topic/", "Test", None, None);
        repository.add_comment(&new_comment);

        let result = repository.comment_with_id(new_comment.id);

        let returned_comment = result.expect("expected a comment");
        assert_eq!(returned_comment.id, new_comment.id);
        assert_eq!(returned_comment.text, "Test");
    }

    #[test]
    fn comments_can_be_retrieved_by_path() {
        let repository = CommentRepository::for_testing();
        repository.add_comment(&Comment::new("/test-topic/", "First comment", None, None));
        repository.add_comment(&Comment::new("/something-else/", "Second comment", None, None));

        let list = repository.comments_for_path("/test-topic/");

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].text, "First comment");
    }

    #[test]
    fn comments_are_sorted_by_timestamp() {
        let repository = CommentRepository::for_testing();
        let mut c1 = Comment::new("/test-topic/", "Second comment", None, None);
        c1.timestamp = c1.timestamp - Duration::minutes(5);
        repository.add_comment(&c1);
        let c2 = Comment::new("/test-topic/", "Third comment", None, None);
        repository.add_comment(&c2);
        let mut c3 = Comment::new("/test-topic/", "First comment", None, None);
        c3.timestamp = c3.timestamp - Duration::hours(2);
        repository.add_comment(&c3);

        let list = repository.comments_for_path("/test-topic/");

        assert_eq!(list.len(), 3);
        assert_eq!(list[0].text, "First comment");
        assert_eq!(list[1].text, "Second comment");
        assert_eq!(list[2].text, "Third comment");
    }
}
