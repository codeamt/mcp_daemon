use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use anyhow::{Result, Context};

/// Configuration for the MCP Daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    /// General configuration
    #[serde(default)]
    pub general: GeneralConfig,

    /// Server configurations
    #[serde(default)]
    pub servers: Vec<ServerConfig>,

    /// Client configurations
    #[serde(default)]
    pub clients: Vec<ClientConfig>,

    /// Router configuration
    #[serde(default)]
    pub router: RouterConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Data directory
    #[serde(default)]
    pub data_dir: Option<PathBuf>,

    /// Silent mode (no TUI)
    #[serde(default)]
    pub silent_mode: bool,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    pub name: String,

    /// Server URL
    pub url: String,

    /// Transport type
    #[serde(default = "default_transport")]
    pub transport: String,

    /// TLS configuration
    #[serde(default)]
    pub tls: TlsConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether TLS is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to verify the server certificate
    #[serde(default = "default_true")]
    pub verify: bool,

    /// Path to client certificate
    #[serde(default)]
    pub client_cert: Option<PathBuf>,

    /// Path to client key
    #[serde(default)]
    pub client_key: Option<PathBuf>,

    /// Server name for SNI
    #[serde(default)]
    pub server_name: Option<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type
    #[serde(default)]
    pub auth_type: String,

    /// Authentication token
    #[serde(default)]
    pub token: String,
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Client ID
    pub id: String,

    /// Client name
    pub name: String,

    /// Allowed servers
    #[serde(default)]
    pub allowed_servers: Vec<String>,
}

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Default route
    #[serde(default)]
    pub default_route: Option<String>,

    /// Load balancing strategy
    #[serde(default = "default_load_balancing")]
    pub load_balancing: String,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// UI theme
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Refresh rate in milliseconds
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u64,
}


impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            log_level: default_log_level(),
            data_dir: None,
            silent_mode: false,
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        TlsConfig {
            enabled: false,
            verify: true,
            client_cert: None,
            client_key: None,
            server_name: None,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            auth_type: "none".to_string(),
            token: String::new(),
        }
    }
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            default_route: None,
            load_balancing: default_load_balancing(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            theme: default_theme(),
            refresh_rate: default_refresh_rate(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_transport() -> String {
    "http2".to_string()
}

fn default_true() -> bool {
    true
}

fn default_load_balancing() -> String {
    "round_robin".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_refresh_rate() -> u64 {
    1000
}

impl Config {
    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        let config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path.as_ref()))?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;

        Ok(())
    }

    /// Get the default configuration file path
    pub fn default_path() -> Option<PathBuf> {
        ProjectDirs::from("io", "mcp", "mcp_daemon").map(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            config_dir.join("daemon.config.json")
        })
    }
}
