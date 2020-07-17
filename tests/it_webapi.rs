extern crate quvyn;
extern crate gotham;
extern crate hyper;
extern crate serde_json;

use std::str;

use gotham::test::{TestClient, TestResponse, TestServer};
use serde_json::{Map, Value};

use quvyn::{webapi, utils};
use gotham::plain::test::TestConnect;


fn client() -> TestClient<TestServer, TestConnect> {
    TestServer::new(webapi::router()).unwrap().client()
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
fn it_can_ping_api() {
    let client = client();

    let response = client.get(&url("/ping")).perform().unwrap();

    assert_eq!(response.status(), 200);
    let obj = as_json_obj(response);
    assert_eq!("ok", obj.get("status").unwrap());
}

