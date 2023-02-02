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

/// # generate the grache config from headers and query parameters
/// * prefers headers if provided
/// ## sets default values if nothing is set
/// * default value for @expiration: 600
/// * default value for @ignore_cookies: false
pub fn get_grache_config(
    headers: &mut HeaderMap,
    query_params: &HashMap<String, String>,
) -> GracheConfig {
    // parse ignore cookies
    let mut ignore_cookies = query_params
        .get("ignoreCookies")
        .and_then(|ic| ic.parse::<bool>().ok())
        .unwrap_or(false);

    ignore_cookies = headers
        .get("Grache_Ignore_Cookies")
        .and_then(|ic| ic.to_str().ok())
        .and_then(|ic| ic.parse::<bool>().ok())
        .unwrap_or(ignore_cookies);

    headers.remove("Grache_Ignore_Cookies");

    // parse expiration
    let mut expiration = query_params
        .get("ignoreCookies")
        .and_then(|ic| ic.parse::<i32>().ok())
        .unwrap_or(600);

    expiration = headers
        .get("Grache_Expiration")
        .and_then(|ex| ex.to_str().ok())
        .and_then(|ex| ex.parse::<i32>().ok())
        .unwrap_or(600);

    headers.remove("Grache_Expiration");
    // return
    GracheConfig {
        ignore_cookies,
        expiration,
    }
}
