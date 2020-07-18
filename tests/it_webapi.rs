extern crate quvyn;
extern crate gotham;
extern crate hyper;
extern crate serde_json;

use std::str;
use gotham::test::{TestClient, TestResponse, TestServer};
use gotham::plain::test::TestConnect;
use serde_json::{Map, Value};
use quvyn::{webapi, utils};
use quvyn::repository::CommentRepository;
use quvyn::comment::Comment;


fn repo(test_name: &str) -> CommentRepository {
    let path = format!("var/it/webapi/{}", test_name);
    CommentRepository::new(&path, true)
}

fn client(repo: CommentRepository) -> TestClient<TestServer, TestConnect> {
    TestServer::new(webapi::router(repo)).unwrap().client()
}

fn url(path: &str) -> String {
    format!("http://testhost{}", path)
}

fn as_json_obj(response: TestResponse) -> Map<String, Value> {
    let body = response.read_utf8_body().unwrap();
    let json_val: Value = utils::from_json(&body);
    json_val.as_object().expect("expected object").clone()
}


#[test]
fn it_ping_api() {
    let client = client(repo("it_ping_api"));

    let response = client.get(&url("/ping")).perform().unwrap();

    assert_eq!(response.status(), 200);
    let obj = as_json_obj(response);
    assert_eq!("ok", obj.get("status").unwrap());
}

#[test]
fn it_get_all_comments() {
    let mut repo = repo("it_get_all_comments");
    repo.save_comment(&Comment::new("/", "First comment"));
    repo.save_comment(&Comment::new("/", "Second comment"));
    let client = client(repo);

    let response = client.get(&url("/comments")).perform().unwrap();

    assert_eq!(response.status(), 200);
    let comments = as_json_obj(response)
        .get("comments").expect("expected comments field")
        .as_array().expect("expected comments to be an array")
        .clone();
    assert_eq!(comments.len(), 2);
}

#[test]
fn it_get_comments_for_topic() {
    let mut repo = repo("it_get_comments_for_topic");
    repo.save_comment(&Comment::new("/1/", "First comment"));
    repo.save_comment(&Comment::new("/2/", "Second comment"));
    repo.save_comment(&Comment::new("/2/", "Third comment"));
    repo.save_comment(&Comment::new("/3/", "Fourth comment"));
    let client = client(repo);

    let response = client.get(&url("/comments?p=%2F2%2F")).perform().unwrap();

    assert_eq!(response.status(), 200);
    let comments = as_json_obj(response)
        .get("comments").expect("expected comments field")
        .as_array().expect("expected comments to be an array")
        .clone();
    assert_eq!(comments.len(), 2);
    let content = comments[0]
        .get("content").expect("expected content field")
        .as_str().expect("expected conversion to str to succeed");
    assert_eq!(content, "Second comment");
}

