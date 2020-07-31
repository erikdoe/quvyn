use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline;
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::builder::DefineSingleRoute;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::*;
use hyper::{Body, Response, StatusCode, Uri};
use serde::Serialize;
use serde_derive::*;
use serde_json::{to_string, Value};

use crate::comment::Comment;
use crate::repository::CommentRepository;
use gotham::handler::HandlerFuture;
use hyper::rt::{Stream, Future};
use futures::future;

pub fn run(repo: CommentRepository, addr: String) {
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(repo));
}

pub fn router(repo: CommentRepository) -> Router {
    let middleware = StateMiddleware::new(repo);
    let pipeline = pipeline::single_middleware(middleware);
    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/ping")
            .to(get_ping);
        route.get("/comments")
            .with_query_string_extractor::<CommentsQueryStringExtractor>()
            .to(get_comments);
        route.post("/comments")
            .to(post_comment);
    })
}


#[derive(Serialize)]
struct PingResponse {
    status: String,
}

fn get_ping(state: State) -> (State, Response<Body>) {
    let response_obj = PingResponse { status: "ok".to_owned() };
    let response = create_json_response(&state, StatusCode::OK, &response_obj).unwrap();
    (state, response)
}


#[derive(Deserialize, StateData, StaticResponseExtender)]
struct CommentsQueryStringExtractor {
    p: Option<String>,
}

#[derive(Serialize, Clone)]
struct CommentListWrapper {
    comments: Vec<Comment>
}

fn get_comments(mut state: State) -> (State, Response<Body>) {
    let query_param = CommentsQueryStringExtractor::take_from(&mut state);
    let repository = CommentRepository::borrow_from(&state);

    let comments = match query_param.p {
        Some(p) => repository.comments_for_path(&p),
        None => repository.all_comments()
    };

    let response_obj = CommentListWrapper { comments };
    let response = create_json_response(&state, StatusCode::OK, &response_obj).unwrap();
    (state, response)
}

fn post_comment(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state).concat2().then(|full_body| {
        // TODO: consider adding explicit error handling for body and UTF-8 problems
        let body_content = String::from_utf8(full_body.unwrap().to_vec()).unwrap();
        let _ = Comment::from_json(&body_content);
        let response = match serde_json::from_str::<Value>(&body_content) {
            Ok(_) => {
                // TODO:
                // We're not using the parsed objects, just parsing first to ensure in a
                // controlled way that the string is parsable. Maybe we should actually use
                // the parsed attributes?
                let comment = Comment::from_json(&body_content);
                let repo = CommentRepository::borrow_from(&state);
                repo.save_comment(&comment);
                create_post_ok_response(&state, true)
            }
            Err(error) => {
                let body = format!("Error parsing JSON document: {}\n", error);
                create_response(&state, StatusCode::BAD_REQUEST, mime::TEXT_PLAIN, body)
            }
        };
        future::ok((state, response))
    });
    Box::new(f)
}

// see https://github.com/ChristophWurst/gotham-serde-json-body-parser/blob/master/src/lib.rs

pub fn create_json_response<S: Serialize>(state: &State, status: StatusCode, data: &S)
                                          -> Result<Response<Body>, serde_json::Error> {
    to_string(data).map(|json_str| {
        create_response(state, status, mime::APPLICATION_JSON, json_str.into_bytes(),
        )
    })
}

fn create_post_ok_response(state: &State, created: bool) -> Response<Body> {
    let (status, response_body) = if created {
        (StatusCode::CREATED, "Created comment\n")
    } else {
        (StatusCode::OK, "Comment already existed\n")
    };
    let mut response = create_response(state, status, mime::TEXT_PLAIN, response_body);
    let location = format!("{}/{}", Uri::borrow_from(&state), "0"); // TODO
    response.headers_mut().insert("Location", location.parse().unwrap());
    response
}
