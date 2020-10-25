use futures::prelude::*;

use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;
use futures::future;


#[derive(Clone, NewMiddleware)]
pub struct CorsMiddleware {
    origin: Option<String>,
}

impl CorsMiddleware {
    pub fn new(origin: &Option<String>) -> Self {
        Self {
            origin: origin.clone()
        }
    }
}

impl Middleware for CorsMiddleware {
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
        where
            Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        let result = chain(state);
        let f = result.and_then(move |(state, mut response)| {
            {
                if let Some(origin) = self.origin {
                    let headers = response.headers_mut();
                    headers.insert("Access-Control-Allow-Origin", origin.parse().unwrap());
                    headers.insert("Access-Control-Allow-Methods", "*".parse().unwrap());
                    headers.insert("Access-Control-Allow-Headers", "*".parse().unwrap());
                }
            };
            future::ok((state, response))
        });

        Box::new(f)
    }
}
