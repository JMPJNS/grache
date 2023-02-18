use anyhow::Result;
use axum::headers::HeaderName;
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Serialize, Deserialize)]
pub struct Headers {
    inner: HashMap<String, Vec<String>>,
}

impl Headers {
    pub fn from_header_map(headers: &hyper::HeaderMap<HeaderValue>) -> Headers {
        let mut header_hashmap = HashMap::new();
        for (k, v) in headers {
            let k = k.as_str().to_owned();
            let v = String::from_utf8_lossy(v.as_bytes()).into_owned();
            header_hashmap.entry(k).or_insert_with(Vec::new).push(v)
        }
        Headers {
            inner: header_hashmap,
        }
    }

    pub fn to_header_map(&self) -> Result<hyper::HeaderMap<HeaderValue>> {
        let mut header_map = HeaderMap::new();
        for (k, v) in self.inner.clone() {
            for s in v {
                header_map.insert(
                    HeaderName::from_str(k.as_str())?,
                    HeaderValue::from_str(s.as_str())?,
                );
            }
        }
        Ok(header_map)
    }
}
