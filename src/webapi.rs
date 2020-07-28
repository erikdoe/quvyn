use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline;
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::builder::DefineSingleRoute;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::*;
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
        route.get("/ping")
            .to(get_ping);
        route.get("/comments")
            .with_query_string_extractor::<CommentsQueryStringExtractor>()
            .to(get_comments);
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
struct CommentListWrapper<'a> {
    comments: Vec<&'a Comment>
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


// see https://github.com/ChristophWurst/gotham-serde-json-body-parser/blob/master/src/lib.rs

pub fn create_json_response<S: Serialize>(state: &State, status: StatusCode, data: &S)
                                          -> Result<Response<Body>, serde_json::Error> {
    to_string(data).map(|json_str| {
        create_response(state, status, mime::APPLICATION_JSON, json_str.into_bytes(),
        )
    })
}
