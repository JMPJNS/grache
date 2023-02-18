use crate::headers::Headers;
use crate::request_context::RequestContext;
use anyhow::Result;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(skip)]
    pub status: StatusCode,
    pub content: String,
    pub headers: Headers,
}

pub async fn post_request(context: &RequestContext) -> Result<Response> {
    let client = reqwest::Client::new();
    let context = context.clone();
    let body = context.body.to_string();
    let mut req = client.post(context.config.url);
    if let Some(headers) = context.headers {
        let headers = headers.to_header_map();
        if headers.is_ok() {
            req = req.headers(headers?);
        }
    }
    if let Some(body) = body {
        req = req.body(body.to_string())
    }
    let res = req.send().await?;
    return Ok(Response {
        status: res.status(),
        headers: Headers::from_header_map(res.headers()),
        content: res.text().await?,
    });
}
