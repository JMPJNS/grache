mod config;

use std::collections::HashMap;
use axum::{
    routing::get,
    Router,
};
use axum::body::Body;
use axum::extract::Query;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/",
                                            // TODO allow get requests aswell
                                get(|| async { (StatusCode::BAD_REQUEST, "Only POST method allowed") })
                                            .post(cache_post)
                                            .options(pass_options)
    );

    axum::Server::bind(&"0.0.0.0:3333".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn cache_post(
    request: Request<Body>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap
) {

}

async fn pass_options(
    request: Request<Body>
) {

}
