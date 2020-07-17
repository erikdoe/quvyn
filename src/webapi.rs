use gotham::router::Router;
use gotham::router::builder::DrawRoutes;
use gotham::router::builder::DefineSingleRoute;
use gotham::router::builder::build_simple_router;
use gotham::state::State;
use gotham::helpers::http::response::create_response;
use hyper::{Body, Response, StatusCode};


pub fn run(addr: String) {
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}

pub fn router() -> Router {
    build_simple_router( |route| {
        route.get("/ping").to(get_ping);
    })
}

fn get_ping(state: State) -> (State, Response<Body>) {
    let body = "{ \"status\": \"ok\" }\n";
    let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
    (state, response)
}

