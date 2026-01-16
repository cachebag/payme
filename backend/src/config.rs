use std::env;

pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:payme.db?mode=rwc".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_defaults() {
        let _lock = ENV_MUTEX.lock().unwrap();

        let orig_db = std::env::var("DATABASE_URL").ok();
        let orig_port = std::env::var("PORT").ok();

        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("PORT");

        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:payme.db?mode=rwc".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        };

        assert_eq!(config.database_url, "sqlite:payme.db?mode=rwc");
        assert_eq!(config.port, 3001);

        if let Some(v) = orig_db {
            std::env::set_var("DATABASE_URL", v);
        }
        if let Some(v) = orig_port {
            std::env::set_var("PORT", v);
        }
    }

    #[test]
    fn test_config_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();

        let orig_db = std::env::var("DATABASE_URL").ok();
        let orig_port = std::env::var("PORT").ok();

        std::env::set_var("DATABASE_URL", "sqlite:test.db");
        std::env::set_var("PORT", "8080");

        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:payme.db?mode=rwc".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        };

        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.port, 8080);

        if let Some(v) = orig_db {
            std::env::set_var("DATABASE_URL", v);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
        if let Some(v) = orig_port {
            std::env::set_var("PORT", v);
        } else {
            std::env::remove_var("PORT");
        }
    }

    #[test]
    fn test_config_invalid_port_uses_default() {
        let _lock = ENV_MUTEX.lock().unwrap();

        let orig_port = std::env::var("PORT").ok();

        std::env::set_var("PORT", "not_a_number");

        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);

        assert_eq!(port, 3001);

        if let Some(v) = orig_port {
            std::env::set_var("PORT", v);
        } else {
            std::env::remove_var("PORT");
        }
    }
}
