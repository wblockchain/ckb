use serde::{Deserialize, Serialize};
use std::{default::Default, path::PathBuf};

const PGSQL: &str = "postgres://";
const SQLITE: &str = "sqlite://";

/// Rich indexer database type.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum DBDriver {
    /// Sqlite config options.
    #[default]
    Sqlite,
    /// Postgres config options.
    Postgres,
}

impl std::fmt::Display for DBDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBDriver::Postgres => write!(f, "{}", PGSQL),
            DBDriver::Sqlite => write!(f, "{}", SQLITE),
        }
    }
}

/// Rich indexer config options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichIndexerConfig {
    /// Rich indexer database type.
    #[serde(default)]
    pub db_type: DBDriver,
    /// The index-r store path, default `data_dir / indexer / sqlite / sqlite.db`,
    /// which will be realized through IndexerConfig::adjust.
    #[serde(default)]
    pub store: PathBuf,
    /// The database name, default `ckb-rich-indexer`.
    #[serde(default = "default_db_name")]
    pub db_name: String,
    /// The database host.
    #[serde(default = "default_db_host")]
    pub db_host: String,
    /// The database port.
    #[serde(default = "default_db_port")]
    pub db_port: u16,
    /// The database user.
    #[serde(default = "default_db_user")]
    pub db_user: String,
    /// The database password.
    #[serde(default = "default_db_password")]
    pub db_password: String,
}

impl Default for RichIndexerConfig {
    fn default() -> Self {
        Self {
            db_type: DBDriver::default(),
            store: PathBuf::default(),
            db_name: default_db_name(),
            db_host: default_db_host(),
            db_port: default_db_port(),
            db_user: default_db_user(),
            db_password: default_db_password(),
        }
    }
}

fn default_db_name() -> String {
    "ckb-rich-indexer".to_string()
}

fn default_db_host() -> String {
    "127.0.0.1".to_string()
}

fn default_db_port() -> u16 {
    8532
}

fn default_db_user() -> String {
    "postgres".to_string()
}

fn default_db_password() -> String {
    "123456".to_string()
}
