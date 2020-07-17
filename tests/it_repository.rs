extern crate quvyn;

use std::{fs};
use quvyn::repository::CommentRepository;
use quvyn::comment::Comment;


#[test]
fn it_stores_and_retrieves_comment() {

    let path = "var/repo/it";

    if fs::metadata(path).is_ok() {
        fs::remove_dir_all(path).expect("Unable to remove storage directory");
    }
    fs::create_dir_all(path).expect("Unable to create storage directory");

    let repo1 = CommentRepository::new(path);
    let original = Comment::new("/some-topic/", "Nice work!");
    repo1.save_comment(&original);

    let mut repo2 = CommentRepository::new(path);
    repo2.load_comments();
    let comments = repo2.all_comments();

    assert_eq!(1, comments.len());
    assert_eq!("Nice work!", comments[0].content);
}

