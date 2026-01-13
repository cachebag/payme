use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_values: Option<String>,
    pub new_values: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditEntry {
    pub fn new(action: &str, entity_type: &str) -> Self {
        Self {
            user_id: None,
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: None,
            old_values: None,
            new_values: None,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }

    pub async fn save(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let old_json = self.old_values.as_ref().map(|v| v.to_string());
        let new_json = self.new_values.as_ref().map(|v| v.to_string());

        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO audit_logs 
            (user_id, action, entity_type, entity_id, old_values, new_values, ip_address, user_agent, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            RETURNING id
            "#,
        )
        .bind(self.user_id)
        .bind(&self.action)
        .bind(&self.entity_type)
        .bind(self.entity_id)
        .bind(old_json)
        .bind(new_json)
        .bind(&self.ip_address)
        .bind(&self.user_agent)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }
}

pub struct AuditLogger {
    pool: SqlitePool,
}

impl AuditLogger {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_by_user(
        &self,
        user_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT id, user_id, action, entity_type, entity_id, 
                   old_values, new_values, ip_address, user_agent, created_at
            FROM audit_logs
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn count_by_user(&self, user_id: i64) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub async fn get_user_activity_summary(
        &self,
        user_id: i64,
        days: i64,
    ) -> Result<Vec<(String, i64)>, sqlx::Error> {
        let summary = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT action, COUNT(*) as count
            FROM audit_logs
            WHERE user_id = ? AND created_at >= datetime('now', '-' || ? || ' days')
            GROUP BY action
            ORDER BY count DESC
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(summary)
    }
}
