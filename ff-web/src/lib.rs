/// HTTP serving for Fractal Farlands.
///
/// The library implements routing, state, etc. for an FF server;
/// the binary starts up a runtime and Warp server to handle requests.
use axum::{routing::get, Router};

pub fn root_routes(_web_rt: tokio::runtime::Handle) -> axum::Router {
    Router::new().route("/", get(hello))
}

async fn hello() -> &'static str {
    "hello, world!"
}
