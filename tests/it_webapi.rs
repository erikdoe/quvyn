extern crate gotham;
extern crate quvyn;
extern crate serde_json;

use std::str;

use gotham::plain::test::TestConnect;
use gotham::test::{TestClient, TestResponse, TestServer};
use serde_json::{json, Map, Value};
use uuid::Uuid;

use quvyn::{utils, webapi};
use quvyn::comment::Comment;
use quvyn::repository::CommentRepository;

fn repo(test_name: &str) -> CommentRepository {
    let path = format!("var/it/webapi/{}", test_name);
    CommentRepository::new(&path, true)
}

fn client(repo: CommentRepository) -> TestClient<TestServer, TestConnect> {
    TestServer::new(webapi::router("vue", &None, repo)).unwrap().client()
}

fn url(path: &str) -> String {
    format!("http://testhost{}", path)
}

fn as_json_obj(response: TestResponse) -> Map<String, Value> {
    let body = response.read_utf8_body().unwrap();
    let json_val: Value = utils::from_json(&body);
    json_val.as_object().expect("expected object").clone()
}

macro_rules! jsome {
    ( $( $x:expr )? ) => {
        $( Some(&json!($x)) )?
    };
}


#[test]
fn it_ping_api() {
    let client = client(repo("it_ping_api"));

    let response = client.get(&url("/ping")).perform().unwrap();

    assert_eq!(200, response.status());
    let obj = as_json_obj(response);
    assert_eq!(jsome!("ok"), obj.get("status"));
}

#[test]
fn it_post_new_comment_and_retrieve_by_id() {
    let client = client(repo("it_post_new_comment_and_retrieve_by_id"));

    let doc = r#"{
        "path": "/1/",
        "text": "Nice work!"
    }"#;
    let response = client.post(url("/comments"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();
    assert_eq!(201, response.status());

    let location = response.headers().get("Location").expect("expected location header").to_str().unwrap();

    let response = client.get(&url(location)).perform().unwrap();
    assert_eq!(200, response.status());
    let obj = as_json_obj(response);
    assert_eq!(jsome!(location.split('/').last().unwrap()), obj.get("id"));
    assert_eq!(jsome!("Nice work!"), obj.get("text"));
}

#[test]
fn it_post_new_comment_returns_display_version() {
    let client = client(repo("it_post_new_comment_returns_display_version"));

    let doc = r#"{
        "path": "/1/",
        "authorName": "Joe Bloggs",
        "authorEmail": "joe@example.org",
        "text": "Nice work!"
    }"#;
    let response = client.post(url("/comments"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();
    assert_eq!(201, response.status());

    let obj = as_json_obj(response);
    assert_eq!(jsome!("Joe Bloggs"), obj.get("authorName"));
    assert_eq!(None, obj.get("authorEmail"));
}

#[test]
fn it_returns_400_when_posting_invalid_json() {
    let client = client(repo("it_returns_400_when_posting_unparsable_json"));
    let doc = r#"{ "id": 1"#;

    let response = client.post(url("/comments"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(400, response.status());
}

#[test]
fn it_returns_400_when_text_parses_into_nothing() {
    let client = client(repo("it_returns_400_when_text_parses_into_nothing"));
    let doc = r#"{
        "path": "/",
        "text": "<script>foo</script>"
    }"#;

    let response = client.post(url("/comments"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(400, response.status());
}

#[test]
fn it_returns_404_for_non_existing_comment() {
    let client = client(repo("it_returns_404_for_non_existing_comments"));
    let location = format!("/comments/{}", Uuid::new_v4().as_simple());

    let response = client.get(url(&location)).perform().unwrap();

    assert_eq!(404, response.status());
}

#[test]
fn it_delete_comment_and_not_found_by_id() {
    let repo = repo("it_delete_comment_and_not_found_by_id");
    let comment = &Comment::new("/", "First comment", None, None);
    repo.save_comment(comment);
    let client = client(repo);
    let location = format!("/comments/{}", comment.id.as_simple());

    let response = client.delete(&url(&location)).perform().unwrap();
    assert_eq!(200, response.status());

    let response = client.get(&url(&location)).perform().unwrap();
    assert_eq!(404, response.status());
}

#[test]
fn it_returns_404_for_non_existing_comment_when_deleting() {
    let client = client(repo("it_returns_404_for_non_existing_comment_when_deleting"));
    let location = format!("/comments/{}", Uuid::new_v4().as_simple());

    let response = client.delete(url(&location)).perform().unwrap();

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
        .as_array().unwrap().clone();
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
        .as_array().unwrap().clone();
    assert_eq!(2, comments.len());
    assert_eq!(jsome!("/2/"), comments[0].get("path"));
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
        .as_array().unwrap()[0].clone();

    assert_eq!(jsome!(comment.idh), obj.get("idh"));
    assert_eq!(jsome!("<p>First comment</p>\n"), obj.get("textHtml"));
    assert_eq!(jsome!("Joe Bloggs"), obj.get("authorName"));
    assert_eq!(jsome!(comment.author_gravatar), obj.get("authorGravatar"));

    assert_eq!(None, obj.get("id"));
    assert_eq!(None, obj.get("text"));
    assert_eq!(None, obj.get("authorEmail"));
    assert_eq!(None, obj.get("author_email"));
}

#[test]
fn it_previews_markdown() {
    let doc = r#"{ "text": "_foo_" }"#;
    let client = client(repo("it_previews_markdown"));

    let response = client.post(url("/preview"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(200, response.status());
    assert_eq!(mime::TEXT_HTML, response.headers().get("content-type").unwrap().to_str().unwrap());
    let body = response.read_utf8_body().unwrap();
    assert_eq!("<p><em>foo</em></p>", body.trim());
}

#[test]
fn it_previews_malformed_markdown_without_5xx_error() {
    let doc = r#"{ "text": "*_foo<a>*_</a>![bar]bar.jpg)" }"#;
    let client = client(repo("it_previews_malformed_markdown"));

    let response = client.post(url("/preview"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(200, response.status());
}

#[test]
fn it_handles_cors_preflight_requests() {
    let client = client(repo("it_handles_cors_preflight_requests"));

    let response = client.options(url("/comments")).perform().unwrap();
    assert_eq!(204, response.status());

    let response = client.options(url("/preview")).perform().unwrap();
    assert_eq!(204, response.status());
}
