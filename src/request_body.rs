use graphql_parser::query::{parse_query, Definition, OperationDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum RequestBody {
    Unknown,
    JSON(Value),
    Text(String),
    GQL(GQLRequest, GQLType),
}

impl RequestBody {
    /// checks what type of request it is
    /// by looking at the content_type and trying to parse json/graphql
    pub fn new(content: &Option<String>) -> Option<RequestBody> {
        let mut rq = RequestBody::Unknown;

        if let Some(content) = content {
            rq = RequestBody::check_for_text(&content).unwrap_or(rq);
            rq = RequestBody::check_for_json(&content).unwrap_or(rq);
            rq = RequestBody::check_for_gql(&content).unwrap_or(rq);
        } else {
            return None;
        }

        return Some(rq);
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            RequestBody::GQL(req, _req_type) => serde_json::to_string(req).ok(),
            RequestBody::Text(t) => Some(t.to_string()),
            RequestBody::JSON(v) => serde_json::to_string(v).ok(),
            RequestBody::Unknown => None,
        }
    }

    fn check_for_gql(content: &str) -> Option<RequestBody> {
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
                return Some(RequestBody::GQL(
                    gql,
                    if contains_mutation {
                        GQLType::Mutation
                    } else {
                        GQLType::Query
                    },
                ));
            }
        }
        None
    }

    fn check_for_json(content: &str) -> Option<RequestBody> {
        let json = serde_json::from_str(&content);
        if let Some(data) = json.ok() {
            return Some(RequestBody::JSON(data));
        }
        None
    }

    fn check_for_text(content: &str) -> Option<RequestBody> {
        if content.len() > 0 {
            return Some(RequestBody::Text(content.into()));
        }
        None
    }
}

#[test]
fn is_gql_query() {
    let rq = RequestBody::new(
        &String::from(
            r#"
        {
            "query": "query MyQuery { field1, field2 }",
            "operationName": "MyQuery",
            "variables": {}
        }
        "#,
        )
        .into(),
    );
    let rq = rq.unwrap();
    let is_query = match rq {
        RequestBody::GQL(r, t) => match t {
            GQLType::Query => true,
            _ => false,
        },
        _ => {
            panic!("Not a GQL context, got {:?} instead", rq);
        }
    };
    assert_eq!(is_query, true)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GQLRequest {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: Value,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GQLType {
    Query,
    Mutation,
}
