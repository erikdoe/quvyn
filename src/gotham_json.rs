extern crate gotham;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use futures_util::{future, FutureExt};

use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_response;
use gotham::handler::{HandlerError};
use gotham::hyper::{body, Body, Response, StatusCode};

use serde::{Serialize};
use serde::de::{DeserializeOwned};


pub fn take_json_body<'de, T: 'de>(mut state: State) -> Pin<Box<dyn Future<Output=Result<(State, T), (State, HandlerError)>> + Send + 'de>>
    where T: Sized + Send + DeserializeOwned {

    let f = body::to_bytes(Body::take_from(&mut state)).then(|result|
        match result {
            Ok(as_bytes) => {
                let as_string = String::from_utf8(as_bytes.to_vec()).unwrap(); // TODO: will this be a 500 for invalid UTF?
                match serde_json::from_str::<T>(&as_string) {
                    Ok(obj) => future::ok((state, obj)),
                    Err(e) => future::err((state, HandlerError::from(e).with_status(StatusCode::BAD_REQUEST))),
                }
            },
            Err(e) => {
                future::err((state, e.into())) // TODO: should this be a bad request, too?
            }
        }
    );
    f.boxed()
}


pub fn create_json_response<S: Serialize>(state: &State, status: StatusCode, data: &S)
                                          -> Result<Response<Body>, serde_json::Error> {
    create_json_response_with_headers(state, status, HashMap::new(), data)
}

pub fn create_json_response_with_headers<S: Serialize>(state: &State, status: StatusCode, headers: HashMap<&'static str, String>, data: &S)
                                                       -> Result<Response<Body>, serde_json::Error> {
    serde_json::to_string(data).map(|json_str| {
        let mut response = create_response(state, status, mime::APPLICATION_JSON, json_str.into_bytes());
        for (key, value) in headers {
            response.headers_mut().insert(key, value.parse().unwrap());
        }
        response
    })
}
