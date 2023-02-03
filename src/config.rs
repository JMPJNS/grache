use axum::http::HeaderMap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GracheConfig {
    /// * represents, in seconds, how long it should be cached for
    /// * set to 0 to bypass cache
    expiration: i32,
    /// if set to true, ignores the Cookie header for this request
    ignore_cookies: bool,
}

impl GracheConfig {
    /// # generate the grache config from headers and query parameters
    /// * prefers headers if provided
    /// ## sets default values if nothing is set
    /// * default value for @expiration: 600
    /// * default value for @ignore_cookies: false
    pub fn new(headers: &mut HeaderMap, query_params: &HashMap<String, String>) -> GracheConfig {
        let ignore_cookies = GracheConfig::get_option(
            headers,
            query_params,
            "Grache_Ignore_Cookies",
            "ignoreCookies",
            false,
        );
        let expiration = GracheConfig::get_option(
            headers,
            query_params,
            "Grache_Expiration",
            "expiration",
            600,
        );

        return GracheConfig {
            ignore_cookies,
            expiration,
        };
    }

    fn get_option<T>(
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
