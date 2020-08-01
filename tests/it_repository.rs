extern crate quvyn;

use quvyn::comment::Comment;
use quvyn::repository::CommentRepository;


fn repo(test_name: &str, reset: bool) -> CommentRepository {
    let path = format!("var/it/repository/{}", test_name);
    CommentRepository::new(&path, reset)
}


#[test]
fn it_stores_and_retrieves_comment() {

    let repo1 = repo("it_stores_and_retrieves_comment", true);
    let original = Comment::new("/some-topic/", "Nice work!", None, None);
    repo1.save_comment(&original);

    let repo2 = repo("it_stores_and_retrieves_comment", false);
    repo2.load_all_comments();
    let comments = repo2.all_comments();

    assert_eq!(1, comments.len());
    assert_eq!("Nice work!", comments[0].content);
}

#[test]
fn it_saving_a_comment_adds_it_to_the_list() {

    let repo = repo("it_saving_a_comment_adds_it_to_the_list", true);
    let original = Comment::new("/some-topic/", "Nice work!", None, None);
    repo.save_comment(&original);

    let comments = repo.all_comments();

    assert_eq!(1, comments.len());
    assert_eq!("Nice work!", comments[0].content);
}
