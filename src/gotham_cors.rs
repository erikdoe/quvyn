use std::pin::Pin;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;


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
    fn call<Chain>(self, state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
        where
            Chain: FnOnce(State) -> Pin<Box<HandlerFuture>>,
    {
        let result = chain(state);
        let f = async move {
            let (state, mut response) = result.await?;
            if let Some(origin) = self.origin {
                let headers = response.headers_mut();
                headers.insert("Access-Control-Allow-Origin", origin.parse().unwrap());
                headers.insert("Access-Control-Allow-Methods", "*".parse().unwrap());
                headers.insert("Access-Control-Allow-Headers", "*".parse().unwrap());
                headers.insert("Access-Control-Expose-Headers", "location".parse().unwrap());
            };
            Ok((state, response))
        };

        Box::pin(f)
    }
}
