mod config;
mod request_body;
mod request_context;
mod http;

use crate::config::{GracheConfig};
use crate::request_body::RequestBody;
use axum::body::Body;
use axum::extract::Query;
use axum::headers::ContentType;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::{routing::get, Router, TypedHeader, extract};
use std::collections::HashMap;
use std::hash::Hash;
use tower_cookies::{CookieManagerLayer, Cookies};
use crate::http::post_request;
use crate::request_context::RequestContext;

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/",
        get(handle_get).post(handle_post), // .options(pass_options),
    ).layer(CookieManagerLayer::new());

    axum::Server::bind(&"0.0.0.0:3333".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn handle_get(
    Query(params): Query<HashMap<String, String>>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    mut headers: HeaderMap,
    cookies: Cookies,
    body: Option<String>,
) -> String {
    let config = GracheConfig::new(&mut headers, &params);

    let context = RequestContext::new(RequestBody::Unknown, cookies, config, headers);
    let hash = context.cache_key();

    "aughh".into()
}

#[axum_macros::debug_handler]
async fn handle_post(
    Query(params): Query<HashMap<String, String>>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    mut headers: HeaderMap,
    cookies: Cookies,
    extract::OriginalUri(uri): extract::OriginalUri,
    body: Option<String>,
) -> String {
    let config = GracheConfig::new(&mut headers, &params);

    let request_body = RequestBody::new(&content_type, &body);
    if request_body.is_none() {
        // TODO: just pass through the request without caching it
        return "not yet implemented".into()
    }
    let body = request_body.unwrap();

    let context = RequestContext::new(body, cookies, config, headers);
    let hash = context.cache_key();
    let res = post_request(context).await;
    println!("{:?}", res);

    "aughh".into()
}

// async fn pass_options(request: Request<Body>) {}
