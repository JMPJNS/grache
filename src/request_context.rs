use crate::config::GracheConfig;
use crate::headers::Headers;
use crate::request_body::{GQLType, RequestBody};
use anyhow::{anyhow, Result};
use axum::headers::HeaderValue;
use axum::http::HeaderMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tower_cookies::Cookies;
use url::{Url};

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub body: RequestBody,
    pub cookies: Cookies,
    pub config: GracheConfig,
    pub headers: Option<Headers>,
}

impl RequestContext {
    pub fn new(
        body: RequestBody,
        cookies: Cookies,
        config: GracheConfig,
        headers: HeaderMap,
    ) -> RequestContext {
        let mut context = RequestContext {
            body,
            cookies,
            config,
            headers: None,
        };
        if context.set_headers(&headers).is_err() {
            context.headers = Some(Headers::from_header_map(&headers));
        }
        return context;
    }

    pub fn set_headers(&mut self, headers: &HeaderMap) -> Result<()> {
        let mut parsed_headers = headers.clone();
        let host = Url::parse(&self.config.url)?;
        let host = host
            .host_str()
            .ok_or(anyhow!("extracting host string failed"))?;

        // TODO figure out a way how to do this without having it being error prone
        // by needing to check which headers grache uses manuall
        // probably using an enum for grache headers
        parsed_headers.remove("Grache-Ignore-Auth");
        parsed_headers.remove("Grache-Expiration");
        parsed_headers.remove("Grache-Cache-Mutations");
        parsed_headers.remove("Grache-Url");

        parsed_headers.remove("Host");
        parsed_headers.insert("Host", HeaderValue::from_str(host)?);
        parsed_headers.remove("Content-Length");
        parsed_headers.remove("Accept-Encoding");
        parsed_headers.insert("Accept-Encoding", HeaderValue::from_str("gzip".into())?);
        self.headers = Some(Headers::from_header_map(&parsed_headers));
        Ok(())
    }
}

impl Hash for RequestContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.config.url.hash(state);

        if !self.config.ignore_auth {
            // TODO: add a way to specify what it uses for authentication instead of hard-coding it
            let session = self.cookies.get("session");
            let session_sig = self.cookies.get("session_sig");
            if let (Some(session), Some(session_sig)) = (session, session_sig) {
                session.value().hash(state);
                session_sig.value().hash(state);
            }
        }

        match &self.body {
            RequestBody::GQL(gql, request_type) => {
                if self.config.cache_mutations || matches!(request_type, GQLType::Query) {
                    gql.query.hash(state);
                    gql.variables.to_string().hash(state);
                }
            }
            RequestBody::JSON(v) => {
                v.to_string().hash(state);
            }
            RequestBody::Text(v) => {
                v.hash(state);
            }
            RequestBody::Unknown => {}
        };
    }
}

impl RequestContext {
    pub fn cache_key(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}
