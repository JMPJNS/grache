use axum::body::Body;
use axum::headers::ContentType;
use axum::http::HeaderMap;
use graphql_parser::query::{parse_query, Definition, OperationDefinition, ParseError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub enum RequestContext {
    GQL(GQLRequest, GQLType),
    Unknown,
}

impl RequestContext {
    /// checks what type of request it is
    /// by looking at the content_type and trying to parse json/graphql
    pub fn get_type(content_type: &ContentType, content: &str) -> RequestContext {
        if content_type != &ContentType::json() {
            return RequestContext::Unknown;
        }

        // check if graphql request
        let gql_request: Option<GQLRequest> = serde_json::from_str(content).unwrap_or_default();
        if let Some(gql) = gql_request {
            // this gets set to true inside the match block
            // if the gql request is of type mutation
            let mut is_mutation = false;
            let is_gql = match parse_query::<&str>(&gql.query) {
                // TODO check all of the definitions instead of just the first one
                Ok(query) => match &query.definitions[0] {
                    Definition::Operation(o) => match o {
                        OperationDefinition::Mutation(_) => {
                            is_mutation = true;
                            true
                        }
                        OperationDefinition::Query(_) => true,
                        _ => false,
                    },
                    _ => false,
                },
                Err(_) => false,
            };
            if is_gql {
                return RequestContext::GQL(
                    gql,
                    if is_mutation {
                        GQLType::Mutation
                    } else {
                        GQLType::Query
                    },
                );
            }
        };

        return RequestContext::Unknown;
    }
}

#[test]
fn is_query() {
    use std::mem::discriminant;

    let rq = RequestContext::get_type(
        &ContentType::json(),
        r#"
        {
            "query": "query MyQuery { field1, field2 }",
            "operationName": "MyQuery",
            "variables": {}
        }
        "#,
    );
    let is_query = match rq {
        RequestContext::GQL(r, t) => match t {
            GQLType::Query => true,
            _ => false,
        },
        _ => false,
    };
    assert_eq!(is_query, true)
}

#[derive(Serialize, Deserialize)]
pub struct GQLRequest {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: Value,
}

pub enum GQLType {
    Query,
    Mutation,
}
