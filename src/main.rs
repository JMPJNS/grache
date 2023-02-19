mod config;
mod headers;
mod http;
mod request_body;
mod request_context;

use crate::config::GracheConfig;
use crate::http::{post_request, Response};
use crate::request_body::RequestBody;
use crate::request_context::RequestContext;

use redis::Commands;
use axum::extract::{Query, State};

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, AppendHeaders};
use axum::{routing::get, Router};
use std::collections::HashMap;


use std::{env};
use tower_cookies::{CookieManagerLayer, Cookies};

#[tokio::main]
async fn main() {

    let redis_client = redis::Client::open(env::var("REDIS_URL").unwrap_or(String::from("redis://127.0.0.1/"))).unwrap();

    let app = Router::new()
        .route(
            "/",
            get(handle).post(handle),
        )
        .layer(CookieManagerLayer::new())
        .with_state(redis_client);

    axum::Server::bind(&"0.0.0.0:3333".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn handle(
    State(redis_client): State<redis::Client>,
    Query(params): Query<HashMap<String, String>>,
    mut headers: HeaderMap,
    cookies: Cookies,
    body: Option<String>,
) -> Result<impl IntoResponse, (StatusCode, String)>{
    let mut con = redis_client.get_connection().map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Connection to Redis failed: {:?}", e)
    ))?;

    let config = GracheConfig::new(&mut headers, &params);

    let body = RequestBody::new(&body).ok_or((
        StatusCode::BAD_REQUEST,
        String::from("Invalid Request Body"),
    ))?;

    let context = RequestContext::new(body, cookies, config, headers);
    let cache_key = context.cache_key();

    let res: Option<Response> = con.get(cache_key).ok();
    let cache_hit = res.is_some();

    let mut res = if let Some(res) = res {
        res
    } else {
        let res = post_request(&context)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        res
    };

    res.headers.set_cache_hit(cache_hit);
    
    if !cache_hit && res.status.unwrap_or(StatusCode::NOT_FOUND) == StatusCode::OK {
        let _ = con.set::<u64, std::option::Option<std::string::String>, redis::Value>(cache_key, res.to_string());
        let _ = con.expire::<u64, redis::Value>(cache_key, context.config.expiration);
    }

    let response_headers = res.headers.to_response_headers()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    return Ok((AppendHeaders(response_headers), res.content))
}
