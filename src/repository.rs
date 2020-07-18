use glob::{glob};
use gotham_derive::*;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use crate::comment::Comment;
use crate::utils::calculate_hash;


#[derive(Clone, StateData)]
pub struct CommentRepository {
   root: String,
   comments: Vec<Comment>,
}


impl CommentRepository {

   pub fn new(path: &str, reset: bool) -> CommentRepository {
      if reset {
         CommentRepository::remove_storage_directory(path);
      }
      CommentRepository::create_storage_directory(path);
      CommentRepository {
         root: path.to_owned(),
         comments: vec![]
      }
   }

   pub fn load_comments(&mut self)  {
      for entry in glob(&format!("{}/**/*.json", self.root)).unwrap() {
         match entry {
            Ok(path) => self.comments.push(CommentRepository::load_comment_from_file(&path.display().to_string())), // TODO
            Err(_) => {} // TODO
         }
      }
   }

   pub fn all_comments(&self) -> &Vec<Comment> {
      &self.comments
   }

   pub fn add_comment(&mut self, comment: &Comment) {
      self.comments.push(comment.clone());
   }

   pub fn save_comment(&mut self, comment: &Comment) {
      let path = self.path_for_comment(&comment);
      fs::create_dir_all(path).expect("Unable to create storage directory");
      let filename = self.filename_for_comment(&comment);
      CommentRepository::save_comment_to_file(comment, &filename);
      self.add_comment(comment); // TODO: there is no test to check that this happens after saving
   }


   // helper functions to interact with file system; maybe should move into their own module

   fn create_storage_directory(path: &str) {
      fs::create_dir_all(path).expect(&format!("Unable to create directory at {}", path)); // TODO: expect
   }

   fn remove_storage_directory(path: &str) {
      if fs::metadata(path).is_ok() {
         match fs::remove_dir_all(path) {
            Ok(_) => {}
            Err(error) => { panic!("Failed to remove storage directory at {}: {}", path, error) }
         }
      }
   }

   fn load_comment_from_file(filename: &str) -> Comment {
      println!("Loading comment from file: {}", filename);
      let mut file = File::open(filename).expect(&format!("Failed to open file {}", filename));
      let mut contents = String::new();
      file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", filename));
      Comment::from_json(&contents)
   }

   fn save_comment_to_file(comment: &Comment, filename: &str) {
      println!("Saving comment to file: {}", filename);
      let mut file = File::create(filename).expect(&format!("Failed to create file {}", filename));
      let _result = file.write_all(comment.to_json().as_ref());
   }

   fn path_for_comment(&self, comment: &Comment) -> String
   {
      // we assume topic id is sanitised to include slashes and no query parameters and anchors
      let regex = Regex::new(r"[^0-9A-Za-z/-]").unwrap();
      let safe_id = regex.replace_all(&comment.topic_id, "");
      format!("{}{}", self.root, safe_id)
   }

   fn filename_for_comment(&self, comment: &Comment) -> String
   {
      format!("{}{}.json", self.path_for_comment(comment), calculate_hash(&comment.content))
   }

}



#[cfg(test)]
mod tests {
   use super::*;

   impl CommentRepository {
      fn for_testing() -> CommentRepository {
         CommentRepository {
            root: "/r".to_owned(),
            comments: vec![]
         }
      }
   }


   #[test]
   fn adding_comment_makes_it_available_in_list() {
      let mut repository = CommentRepository::for_testing();
      let comment = Comment::new("/test-topic/", "Test");
      repository.add_comment(&comment);

      let list = repository.all_comments();

      assert_eq!(1, list.len());
   }

   #[test]
   fn storage_path_for_comment_concatenates_core_parts() {
      let repository = CommentRepository::for_testing();
      let comment = Comment::new("/test-topic/", "Test");

      let path = repository.path_for_comment(&comment);

      assert_eq!("/r/test-topic/", path);
   }

   #[test]
   fn storage_path_for_comment_removes_non_ascii_chars() {
      let repository = CommentRepository::for_testing();
      let comment = Comment::new("/t&*e#öst/", "Test");

      let path = repository.path_for_comment(&comment);

      assert_eq!("/r/test/", path);
   }

   #[test]
   fn filename_for_comment_hashes_content() {
      let repository = CommentRepository::for_testing();
      let comment = Comment::new("/test-topic/", "Test");

      let path = repository.filename_for_comment(&comment);

      assert_eq!(format!("/r/test-topic/{}.json", calculate_hash(&comment.content)), path);

   }

}
