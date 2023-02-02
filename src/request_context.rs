use axum::body::Body;
use axum::headers::ContentType;
use axum::http::HeaderMap;
use graphql_parser::query::{parse_query, Definition, OperationDefinition, ParseError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum RequestContext {
    Unknown,
    JSON(Value),
    GQL(GQLRequest, GQLType),
}

impl RequestContext {
    /// checks what type of request it is
    /// by looking at the content_type and trying to parse json/graphql
    pub fn new(content_type: &ContentType, content: &str) -> RequestContext {
        let mut rq = RequestContext::Unknown;

        rq = RequestContext::check_for_json(&content_type, &content).unwrap_or(rq);
        rq = RequestContext::check_for_gql(&content_type, &content).unwrap_or(rq);

        return rq;
    }

    fn check_for_gql(content_type: &ContentType, content: &str) -> Option<RequestContext> {
        // check if graphql request
        let gql_request: Option<GQLRequest> = serde_json::from_str(content).unwrap_or_default();
        if let Some(gql) = gql_request {
            // this gets set to true inside the match block
            // if the gql request is of type mutation
            let mut contains_mutation = false;
            let is_gql = match parse_query::<&str>(&gql.query) {
                // TODO check all of the definitions instead of just the first one
                Ok(query) => match &query.definitions[0] {
                    Definition::Operation(o) => match o {
                        OperationDefinition::Mutation(_) => {
                            contains_mutation = true;
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
                return Some(RequestContext::GQL(
                    gql,
                    if contains_mutation {
                        GQLType::Mutation
                    } else {
                        GQLType::Query
                    },
                ))
            }
        }
        None
    }
    fn check_for_json(content_type: &ContentType, content: &str) -> Option<RequestContext> {
        if content_type == &ContentType::json() {
            let json = serde_json::from_str(&content);
            if let Some(data) = json.ok() {
                return Some(RequestContext::JSON(data))
            }
        }
        None
    }
}

#[test]
fn is_gql_query() {
    use std::mem::discriminant;

    let rq = RequestContext::new(
        &ContentType::json(),
        r#"
        {
            "quey": "query MyQuery { field1, field2 }",
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
        _ => {
            panic!("Not a GQL context, got {:?} instead", rq);
        },
    };
    assert_eq!(is_query, true)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GQLRequest {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: Value,
}

#[derive(Debug)]
pub enum GQLType {
    Query,
    Mutation,
}
