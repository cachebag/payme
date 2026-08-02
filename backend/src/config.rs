use std::env;

pub struct Config {
    pub bind: String,
    pub database_url: String,
    pub static_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            bind: env::var("PAYME_BIND").unwrap_or_else(|_| "127.0.0.1:3001".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:payme.db?mode=rwc".to_string()),
            static_dir: env::var("PAYME_STATIC_DIR")
                .unwrap_or_else(|_| "frontend/dist".to_string()),
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

        let orig_bind = std::env::var("PAYME_BIND").ok();
        let orig_db = std::env::var("DATABASE_URL").ok();
        let orig_static_dir = std::env::var("PAYME_STATIC_DIR").ok();

        std::env::remove_var("PAYME_BIND");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("PAYME_STATIC_DIR");

        let config = Config::from_env();

        assert_eq!(config.bind, "127.0.0.1:3001");
        assert_eq!(config.database_url, "sqlite:payme.db?mode=rwc");
        assert_eq!(config.static_dir, "frontend/dist");

        if let Some(v) = orig_bind {
            std::env::set_var("PAYME_BIND", v);
        }
        if let Some(v) = orig_db {
            std::env::set_var("DATABASE_URL", v);
        }
        if let Some(v) = orig_static_dir {
            std::env::set_var("PAYME_STATIC_DIR", v);
        }
    }

    #[test]
    fn test_config_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();

        let orig_bind = std::env::var("PAYME_BIND").ok();
        let orig_db = std::env::var("DATABASE_URL").ok();
        let orig_static_dir = std::env::var("PAYME_STATIC_DIR").ok();

        std::env::set_var("PAYME_BIND", "127.0.0.1:8080");
        std::env::set_var("DATABASE_URL", "sqlite:test.db");
        std::env::set_var("PAYME_STATIC_DIR", "/tmp/payme-dist");

        let config = Config::from_env();

        assert_eq!(config.bind, "127.0.0.1:8080");
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.static_dir, "/tmp/payme-dist");

        if let Some(v) = orig_bind {
            std::env::set_var("PAYME_BIND", v);
        } else {
            std::env::remove_var("PAYME_BIND");
        }
        if let Some(v) = orig_db {
            std::env::set_var("DATABASE_URL", v);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
        if let Some(v) = orig_static_dir {
            std::env::set_var("PAYME_STATIC_DIR", v);
        } else {
            std::env::remove_var("PAYME_STATIC_DIR");
        }
    }
}
