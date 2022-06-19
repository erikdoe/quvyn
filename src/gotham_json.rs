extern crate gotham;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use futures_util::{future, FutureExt, TryFutureExt};

use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_response;
use gotham::handler::{HandlerError};
use gotham::hyper::{body, Body, Response, StatusCode};

use serde::{Serialize};
use serde::de::{DeserializeOwned};
use serde_json::{from_str, to_string};


// pub trait JSONBody {
//     fn json<'de, T: 'de>(self) -> Box<dyn Future<Item=(State, T), Error=(State, HandlerError)> + Send + 'de>
//         where T: DeserializeOwned + Send;
// }
//
// impl JSONBody for State {
//     fn json<'de, T: 'de>(mut self) -> Box<dyn Future<Item=(State, T), Error=(State, HandlerError)> + Send + 'de>
//         where T: DeserializeOwned + Send,
//     {
//         let body = Body::take_from(&mut self);
//         let f = body.concat2()
//             .map_err(|err| Error::from(err))
//             .then(|res| match res {
//                 Ok(body) => {
//                     let json = String::from_utf8(body.to_vec()).unwrap();
//                     match from_str(&json) {
//                         Ok(parsed) => Ok((self, parsed)),
//                         Err(err) => Err((self, Error::from(err))),
//                     }
//                 }
//                 Err(err) => Err((self, err)),
//             })
//             .map_err(|(state, err)| {
//                 (
//                     state,
//                     HandlerError::with_status(
//                         err.compat().into_handler_error(),
//                         StatusCode::BAD_REQUEST,
//                     ),
//                 )
//             });
//
//         Box::new(f)
//     }
// }

fn get_body(state: &mut State) -> Pin<Box<dyn Future<Output = Result<String, gotham::hyper::Error>> + Send>> {
    let f = body::to_bytes(Body::take_from(state)).and_then(|as_bytes| {
        future::ok(String::from_utf8(as_bytes.to_vec()).unwrap())
    });
    f.boxed()
}

pub fn get_json_body<'de, T: 'de>(state: &mut State) -> Pin<Box<dyn Future<Output = Result<T, HandlerError>> + Send + 'de>>
    where T: Sized + Send + DeserializeOwned {
    let f = get_body(state).then(|result| match result {
        Ok(json) => {
            match from_str::<T>(&json) {
                Ok(obj) => future::ok(obj),
                Err(e) => future::err(e.into()),
            }
        }
        Err(e) => {
            future::err(e.into())
        }
    });
    f.boxed()
}

pub fn create_json_response<S: Serialize>(state: &State, status: StatusCode, data: &S)
                                          -> Result<Response<Body>, serde_json::Error> {
    create_json_response_with_headers(state, status, HashMap::new(), data)
}

pub fn create_json_response_with_headers<S: Serialize>(state: &State, status: StatusCode, headers: HashMap<&'static str, String>, data: &S)
                                          -> Result<Response<Body>, serde_json::Error> {
    to_string(data).map(|json_str| {
        let mut response = create_response(state, status, mime::APPLICATION_JSON, json_str.into_bytes());
        for (key, value) in headers {
            response.headers_mut().insert(key, value.parse().unwrap());
        }
        response
    })
}
