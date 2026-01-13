use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use http_body_util::BodyExt;
use std::sync::Arc;

use crate::cache::CacheManager;

pub struct CacheState {
    pub manager: Arc<CacheManager>,
}

pub async fn cache_middleware(
    State(state): State<Arc<CacheState>>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() != Method::GET {
        return next.run(request).await;
    }

    let cache_key = generate_cache_key(&request);

    if let Some(cached_body) = state.manager.get_response(&cache_key).await {
        let mut response = Response::new(Body::from(cached_body));
        *response.status_mut() = StatusCode::OK;
        response
            .headers_mut()
            .insert("x-cache-status", HeaderValue::from_static("HIT"));
        response
            .headers_mut()
            .insert("content-type", HeaderValue::from_static("application/json"));
        return response;
    }

    let response = next.run(request).await;

    if should_cache(&response) {
        let (parts, body) = response.into_parts();

        match body.collect().await {
            Ok(collected) => {
                let body_bytes = collected.to_bytes();
                let body_clone = body_bytes.clone();

                let cache_key_clone = cache_key.clone();
                let manager = Arc::clone(&state.manager);
                tokio::spawn(async move {
                    manager
                        .put_response(cache_key_clone, body_clone.to_vec())
                        .await;
                });

                let mut new_response = Response::from_parts(parts, Body::from(body_bytes));
                new_response
                    .headers_mut()
                    .insert("x-cache-status", HeaderValue::from_static("MISS"));
                new_response
            }
            Err(_) => Response::from_parts(parts, Body::from("")),
        }
    } else {
        response
    }
}

fn generate_cache_key(request: &Request) -> String {
    let path = request.uri().path();
    let query = request.uri().query().unwrap_or("");

    let user_id = request
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .map(|c| c.sub.to_string())
        .unwrap_or_else(|| "anon".to_string());

    format!("{}:{}:{}", user_id, path, query)
}

fn should_cache(response: &Response) -> bool {
    if !response.status().is_success() {
        return false;
    }

    if let Some(cache_control) = response.headers().get("cache-control") {
        if let Ok(value) = cache_control.to_str() {
            if value.contains("no-cache") || value.contains("no-store") {
                return false;
            }
        }
    }

    true
}
