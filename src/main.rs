mod config;
mod headers;
mod http;
mod request_body;
mod request_context;

use crate::config::GracheConfig;
use crate::http::post_request;
use crate::request_body::RequestBody;
use crate::request_context::RequestContext;
use axum::body::Body;
use axum::extract::Query;
use axum::headers::{ContentType, HeaderName};
use axum::http::{HeaderMap, HeaderValue, Method, Request, StatusCode, response};
use axum::response::{IntoResponse, Response, AppendHeaders};
use axum::{extract, routing::get, Router, TypedHeader};
use std::collections::HashMap;
use std::hash::Hash;
use std::vec;
use tower_cookies::{CookieManagerLayer, Cookies};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/",
            get(handle).post(handle), // .options(pass_options),
        )
        .layer(CookieManagerLayer::new());

    axum::Server::bind(&"0.0.0.0:3333".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn handle(
    Query(params): Query<HashMap<String, String>>,
    mut headers: HeaderMap,
    cookies: Cookies,
    body: Option<String>,
) -> Result<impl IntoResponse, (StatusCode, String)>{
    let config = GracheConfig::new(&mut headers, &params);

    let body = RequestBody::new(&body).ok_or((
        StatusCode::BAD_REQUEST,
        String::from("Invalid Request Body"),
    ))?;

    println!("Handling {:?}", body);

    let context = RequestContext::new(body, cookies, config, headers);
    // let hash = context.cache_key();
    let res = post_request(&context)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response_headers = res.headers.to_response_headers()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    return Ok((AppendHeaders(response_headers?), res.content))
}
