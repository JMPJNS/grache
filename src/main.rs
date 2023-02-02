mod config;
mod request_context;

use crate::config::get_grache_config;
use crate::request_context::RequestContext;
use axum::body::Body;
use axum::extract::Query;
use axum::headers::ContentType;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::{routing::get, Router, TypedHeader};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route(
        "/",
        // TODO allow get requests aswell
        get(|| async { (StatusCode::BAD_REQUEST, "Only POST method allowed") }).post(cache_post), // .options(pass_options),
    );

    axum::Server::bind(&"0.0.0.0:3333".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn cache_post(
    Query(params): Query<HashMap<String, String>>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    mut headers: HeaderMap,
    body: String,
) -> &'static str {
    // config for this request
    let config = get_grache_config(&mut headers, &params);

    // check what type of request it is
    let request_context = RequestContext::new(&content_type, &body);
    match request_context {
        RequestContext::GQL(gql, request_type) => {

        }
        RequestContext::JSON(val) => {

        }
        // always pass through any unknown requests
        _ => {

        }
    }
    "aughh"
}

// async fn pass_options(request: Request<Body>) {}
