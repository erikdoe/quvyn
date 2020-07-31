use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline;
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::builder::DefineSingleRoute;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::*;
use hyper::{Body, Response, StatusCode, Uri};
use hyper::rt::{Future};
use serde_derive::*;

use crate::comment::Comment;
use crate::gotham_json::{create_json_response, JSONBody};
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


fn post_comment(state: State) -> Box<HandlerFuture> {
    Box::new(state.json::<Comment>().and_then(|(state, comment)| {
        let repo = CommentRepository::borrow_from(&state);
        repo.save_comment(&comment);
        let response = create_post_ok_response(&state, true);
        Ok((state, response))
    }))
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
