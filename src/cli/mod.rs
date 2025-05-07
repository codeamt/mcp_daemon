pub mod config;
pub mod tui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// MCP Daemon - A Rust implementation of the Model Context Protocol (MCP)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Enable the Terminal User Interface (TUI)
    #[arg(long)]
    pub tui: bool,

    /// Increase logging verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands for the CLI
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the daemon
    Start {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },

    /// Stop the daemon
    Stop,

    /// Show daemon status
    Status,

    /// Add a new server or client
    Add {
        /// Type of entity to add (server or client)
        #[arg(value_enum)]
        entity_type: EntityType,

        /// Name of the entity
        #[arg(short, long)]
        name: String,

        /// URL of the server (for server entities)
        #[arg(short, long)]
        url: Option<String>,
    },

    /// Remove a server or client
    Remove {
        /// Type of entity to remove (server or client)
        #[arg(value_enum)]
        entity_type: EntityType,

        /// Name of the entity
        name: String,
    },

    /// List servers or clients
    List {
        /// Type of entity to list (server or client)
        #[arg(value_enum)]
        entity_type: EntityType,
    },

    /// Connect to a server
    Connect {
        /// Name of the server
        name: String,
    },

    /// Disconnect from a server
    Disconnect {
        /// Name of the server
        name: String,
    },
}

/// Entity types for the CLI
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum EntityType {
    /// Server entity
    Server,

    /// Client entity
    Client,
}
