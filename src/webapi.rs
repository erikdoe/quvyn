use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline;
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::builder::DefineSingleRoute;
use gotham::router::Router;
use gotham::state::{State, FromState};
use hyper::{Body, Response, StatusCode};
use serde::Serialize;
use serde_derive::*;
use serde_json::to_string;

use crate::comment::Comment;
use crate::repository::CommentRepository;

pub fn run(repo: CommentRepository, addr: String) {
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(repo));
}

pub fn router(repo: CommentRepository) -> Router {
    let middleware = StateMiddleware::new(repo);
    let pipeline = pipeline::single_middleware(middleware);
    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/ping").to(get_ping);
        route.get("/comments").to(get_comments);
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

#[derive(Serialize, Clone)]
struct CommentListWrapper<'a> {
    comments: &'a Vec<Comment>
}

fn get_comments(state: State) -> (State, Response<Body>) {
    let comments = CommentRepository::borrow_from(&state).all_comments();
    let response_obj = CommentListWrapper { comments };
    let response = create_json_response(&state, StatusCode::OK, &response_obj).unwrap();
    (state, response)
}

// see https://github.com/ChristophWurst/gotham-serde-json-body-parser/blob/master/src/lib.rs

pub fn create_json_response<S: Serialize>(state: &State, status: StatusCode, data: &S)
                                          -> Result<Response<Body>, serde_json::Error> {
    to_string(data).map(|json_str| {
        create_response(state, status, mime::APPLICATION_JSON, json_str.into_bytes(),
        )
    })
}
