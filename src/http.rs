use crate::headers::Headers;
use crate::request_context::RequestContext;
use anyhow::Result;
use redis::{FromRedisValue, RedisResult, from_redis_value, ErrorKind, RedisError, ToRedisArgs};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(skip)]
    pub status: Option<StatusCode>,
    pub content: String,
    pub headers: Headers,
}

impl Response {
    pub fn to_string(&self) -> Option<String> {
        serde_json::to_string(&self).ok()
    }
}

impl FromRedisValue for Response {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        let deserialized: Option<Response> = serde_json::from_str(&v).ok();
        if let Some(res) = deserialized {
            return Ok(res)
        } else {
            Err((ErrorKind::TypeError, "Error Deserializing JSON").into())
        }
    }
}

pub async fn post_request(context: &RequestContext) -> Result<Response> {
    let context = context.clone();

    let client = reqwest::Client::new();
    let mut req = client.post(context.config.url);

    if let Some(headers) = context.headers {
        let headers = headers.to_header_map();
        if headers.is_ok() {
            req = req.headers(headers?);
        }
    }

    let body = context.body.to_string();
    if let Some(body) = body {
        req = req.body(body.to_string())
    }

    let res = req.send().await?;
    return Ok(Response {
        status: Some(res.status()),
        headers: Headers::from_header_map(res.headers()),
        content: res.text().await?,
    });
}
