use reqwest::{StatusCode, Error};
use serde::{Deserialize, Serialize};
use crate::request_context::RequestContext;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(skip)]
    pub status: StatusCode,
    pub content: String,
}

pub async fn post_request(context: RequestContext) -> Result<Response, Error> {
    let client = reqwest::Client::new();
    let body = context.body.to_string();
    let mut req = client.post(context.config.url);
    if let Some(headers) = context.headers {
        req = req.headers(headers)
    }
    if let Some(body) = body {
        //TODO: figure out why the sent body is invalid
        req = req.body(body.to_string())
    }
    println!("{:?}", req);
    let res = req.send().await?;
    return Ok(Response {
        status: res.status(),
        content: res.text().await?,
    })
}