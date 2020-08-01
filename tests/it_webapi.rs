extern crate gotham;
extern crate hyper;
extern crate quvyn;
extern crate serde_json;

use std::str;

use gotham::plain::test::TestConnect;
use gotham::test::{TestClient, TestResponse, TestServer};
use serde_json::{Map, Value};
use uuid::Uuid;

use quvyn::{utils, webapi};
use quvyn::comment::Comment;
use quvyn::repository::CommentRepository;

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

    assert_eq!(200, response.status());
    let obj = as_json_obj(response);
    assert_eq!("ok", obj.get("status").unwrap());
}

#[test]
fn it_post_new_comment_and_retrieve_by_id() {
    let client = client(repo("it_post_new_comment_and_retrieve_by_id"));

    let doc = r#"{ "path": "/1/", "content": "Nice work!" }"#;
    let response = client.post(url("/comments"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();
    assert_eq!(201, response.status());

    let location = response.headers()
        .get("Location").expect("expected location header")
        .to_str().unwrap();

    let response = client.get(&url(location)).perform().unwrap();
    assert_eq!(response.status(), 200);
    let obj = as_json_obj(response);
    let id = obj
        .get("id").expect("expected id field")
        .as_str().unwrap();
    assert_eq!(id, location.split('/').last().unwrap());
    let content = obj
        .get("content").expect("expected content field")
        .as_str().unwrap();
    assert_eq!(content, "Nice work!");
}

#[test]
fn it_returns_400_for_unparsable_json() {
    let client = client(repo("it_returns_400_for_unparsable_json"));
    let doc = r#"{ "id": 1"#;

    let response = client.post(url("/comments"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(400, response.status());
}

#[test]
fn it_returns_404_for_non_existing_comment() {
    let client = client(repo("it_returns_404_for_non_existing_comments"));
    let location = format!("/comments/{}", Uuid::new_v4().to_simple());

    let response = client.get(url(&location)).perform().unwrap();

    assert_eq!(404, response.status());
}

#[test]
fn it_get_all_comments() {
    let repo = repo("it_get_all_comments");
    repo.save_comment(&Comment::new("/", "First comment", None, None));
    repo.save_comment(&Comment::new("/", "Second comment", None, None));
    let client = client(repo);

    let response = client.get(&url("/comments")).perform().unwrap();

    assert_eq!(200, response.status());
    let comments = as_json_obj(response)
        .get("comments").expect("expected comments field")
        .as_array().expect("expected comments to be an array")
        .clone();
    assert_eq!(2, comments.len());
}

#[test]
fn it_get_comments_for_topic() {
    let repo = repo("it_get_comments_for_topic");
    repo.save_comment(&Comment::new("/1/", "First comment", None, None));
    repo.save_comment(&Comment::new("/2/", "Second comment", None, None));
    repo.save_comment(&Comment::new("/2/", "Third comment", None, None));
    repo.save_comment(&Comment::new("/3/", "Fourth comment", None, None));
    let client = client(repo);

    let response = client.get(&url("/comments?p=%2F2%2F")).perform().unwrap();

    assert_eq!(200, response.status());
    let comments = as_json_obj(response)
        .get("comments").expect("expected comments field")
        .as_array().expect("expected comments to be an array")
        .clone();
    assert_eq!(2, comments.len());
    let path = comments[0]
        .get("path").expect("expected path field")
        .as_str().unwrap();
    assert_eq!(path, "/2/");
}

#[test]
fn it_comments_for_display_have_limited_fields() {
    let repo = repo("it_comments_for_display_have_limited_fields");
    let comment = Comment::new("/1/", "First comment", Some("Joe Bloggs"), Some("joe@example.org"));
    repo.save_comment(&comment);
    let client = client(repo);

    let response = client.get(&url("/comments")).perform().unwrap();

    assert_eq!(200, response.status());
    let obj = as_json_obj(response)
        .get("comments").expect("expected comments field")
        .as_array().expect("expected comments to be an array")[0]
        .clone();

    let idh = obj
        .get("idh").expect("expected idh field")
        .as_u64().unwrap();
    assert_eq!(idh, comment.idh);
    let html = obj
        .get("contentHtml").expect("expected contentHtml field")
        .as_str().unwrap();
    assert_eq!(html, "<p>First comment</p>\n");
    let author = obj
        .get("authorName").expect("expected authorName field")
        .as_str().unwrap();
    assert_eq!(author, "Joe Bloggs");

    assert_eq!(obj.get("id").is_some(), false);
    assert_eq!(obj.get("content").is_some(), false);
    assert_eq!(obj.get("authorEmail").is_some(), false);
    assert_eq!(obj.get("author_email").is_some(), false);
}
