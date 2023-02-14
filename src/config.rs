use axum::http::HeaderMap;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct GracheConfig {
    /// * represents, in seconds, how long it should be cached for
    /// * set to 0 to bypass cache
    pub expiration: i32,
    /// if set to true, ignores authentication for this request
    pub ignore_auth: bool,
    /// if set to true, also caches graphql mutations not just querries
    pub cache_mutations: bool,
}

impl GracheConfig {
    /// # generate the grache config from headers and query parameters
    /// * prefers headers if provided
    /// ## sets default values if nothing is set
    /// * default value for @expiration: 600
    /// * default value for @ignore_cookies: false
    pub fn new(headers: &mut HeaderMap, query_params: &HashMap<String, String>) -> GracheConfig {
        let ignore_auth = GracheConfig::get_option(
            headers,
            query_params,
            "Grache_Ignore_Auth",
            "ignoreAuth",
            false,
        );
        let expiration = GracheConfig::get_option(
            headers,
            query_params,
            "Grache_Expiration",
            "expiration",
            600,
        );
        let cache_mutations = GracheConfig::get_option(
            headers,
            query_params,
            "Grache_Cache_Mutations",
            "cacheMutations",
            false,
        );

        return GracheConfig {
            ignore_auth,
            expiration,
            cache_mutations,
        };
    }

    fn get_option<T: FromStr>(
        headers: &mut HeaderMap,
        query_params: &HashMap<String, String>,
        header_name: &str,
        query_param_name: &str,
        default: T,
    ) -> T {
        // use query_param first
        let mut option = query_params
            .get(query_param_name)
            .and_then(|ic| ic.parse::<T>().ok())
            .unwrap_or(default);

        // and then override it if the corresponding header is present
        option = headers
            .get(header_name)
            .and_then(|ic| ic.to_str().ok())
            .and_then(|ic| ic.parse::<T>().ok())
            .unwrap_or(option);

        headers.remove(header_name);
        return option;
    }
}
