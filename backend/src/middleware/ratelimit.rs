use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use crate::middleware::auth::Claims;
use crate::ratelimit::{RateLimitResult, RateLimiter};

pub struct RateLimitState {
    pub limiter: Arc<RateLimiter>,
}

pub async fn rate_limit_middleware(
    State(state): State<Arc<RateLimitState>>,
    claims: Option<axum::Extension<Claims>>,
    request: Request,
    next: Next,
) -> Response {
    let ip = extract_ip(&request);

    let result = if let (Some(ip_addr), Some(claims_ext)) = (ip, claims.as_ref()) {
        state
            .limiter
            .check_rate_limit_combined(ip_addr, claims_ext.sub)
            .await
    } else if let Some(ip_addr) = ip {
        state.limiter.check_rate_limit_ip(ip_addr).await
    } else {
        RateLimitResult {
            allowed: true,
            limit: 100,
            remaining: 100,
            retry_after: None,
        }
    };

    if !result.allowed {
        return rate_limit_exceeded_response(result);
    }

    let mut response = next.run(request).await;

    add_rate_limit_headers(response.headers_mut(), &result);

    response
}

fn extract_ip(request: &Request) -> Option<IpAddr> {
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = IpAddr::from_str(first_ip.trim()) {
                    return Some(ip);
                }
            }
        }
    }

    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                return Some(ip);
            }
        }
    }

    request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip())
}

fn add_rate_limit_headers(headers: &mut HeaderMap, result: &RateLimitResult) {
    if let Ok(limit) = result.limit.to_string().parse() {
        headers.insert("x-ratelimit-limit", limit);
    }
    if let Ok(remaining) = result.remaining.to_string().parse() {
        headers.insert("x-ratelimit-remaining", remaining);
    }
}

fn rate_limit_exceeded_response(result: RateLimitResult) -> Response {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        "Rate limit exceeded. Please try again later.",
    )
        .into_response();

    let headers = response.headers_mut();
    add_rate_limit_headers(headers, &result);

    if let Some(retry_after) = result.retry_after {
        if let Ok(secs) = retry_after.as_secs().to_string().parse() {
            headers.insert("retry-after", secs);
        }
    }

    response
}
