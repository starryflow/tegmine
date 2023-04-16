use std::net::SocketAddr;

use axum::routing::get;
use axum::Router;
use tegmine_common::prelude::*;

#[tokio::main]
async fn main() {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> String {
    let w = Wrapper::default();
    w.test().await
}

struct Inner;
impl Inner {
    async fn test_inner(&self) -> String {
        "TestInner".to_owned()
    }
}

struct Wrapper {
    inner: RcRefCell<Inner>,
}
impl Default for Wrapper {
    fn default() -> Self {
        Self {
            inner: rcrefcell!(Inner),
        }
    }
}
unsafe impl Send for Wrapper {}
unsafe impl Sync for Wrapper {}

impl Wrapper {
    async fn test(&self) -> String {
        self.inner.borrow().test_inner().await
    }
}
