use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;

use crate::audit::AuditEntry;
use crate::middleware::auth::Claims;

pub async fn audit_middleware(
    State(pool): State<SqlitePool>,
    claims: Option<axum::Extension<Claims>>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            request
                .extensions()
                .get::<std::net::SocketAddr>()
                .map(|addr| addr.to_string())
        });
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let user_id = claims.as_ref().map(|c| c.sub);

    let response = next.run(request).await;

    if should_audit(method.as_ref(), uri.path(), response.status()) {
        let action = format!("{} {}", method, uri.path());
        let entity_type = extract_entity_type(uri.path());

        let mut entry = AuditEntry::new(&action, &entity_type);

        if let Some(uid) = user_id {
            entry = entry.with_user_id(uid);
        }

        if let Some(ip_addr) = ip {
            entry = entry.with_ip_address(ip_addr);
        }

        if let Some(ua) = user_agent {
            entry = entry.with_user_agent(ua);
        }

        tokio::spawn(async move {
            if let Err(e) = entry.save(&pool).await {
                tracing::error!("Failed to save audit log: {}", e);
            }
        });
    }

    response
}

fn should_audit(method: &str, path: &str, status: StatusCode) -> bool {
    if !status.is_success() && !status.is_redirection() {
        return false;
    }

    if path.starts_with("/api/audit") || path.starts_with("/health") || path.starts_with("/swagger")
    {
        return false;
    }

    matches!(method, "POST" | "PUT" | "DELETE" | "PATCH")
}

fn extract_entity_type(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() >= 2 && parts[0] == "api" {
        return parts[1].to_string();
    }

    "unknown".to_string()
}
