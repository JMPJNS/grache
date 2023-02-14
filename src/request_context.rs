use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use axum::http::HeaderMap;
use tower_cookies::Cookies;
use crate::config::GracheConfig;
use crate::request_body::{GQLType, RequestBody};

pub struct RequestContext {
    pub url: String,
    pub body: RequestBody,
    pub headers: HeaderMap,
    pub cookies: Cookies,
    pub config: GracheConfig,
}

impl Hash for RequestContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);

        if !self.config.ignore_auth {
            // TODO: add a way to specify what it uses for authentication instead of hardcoding it
            let session = self.cookies.get("session");
            let session_sig = self.cookies.get("session");
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
            },
            RequestBody::JSON(v) => {
                v.to_string().hash(state);
            },
            RequestBody::Text(v) => {
                v.hash(state);
            }
            _ => {}
        };
    }
}

impl RequestContext {
    fn cache_key(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}