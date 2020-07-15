use gotham::router::Router;
use gotham::pipeline;
use gotham::router::builder::{build_router, DrawRoutes, DefineSingleRoute};
use gotham::state::State;
use gotham::helpers::http::response::create_response;
use hyper::{Body, Response, StatusCode};


pub fn run(addr: String) {
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}

pub fn router() -> Router {
    let pipeline = pipeline::new_pipeline().build(); // TODO: double-check if necessary
    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/ping").to(get_ping);
    })
}

fn get_ping(state: State) -> (State, Response<Body>) {
    let body = "{ \"hello\": \"world\" }\n";
    let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
    (state, response)
}
