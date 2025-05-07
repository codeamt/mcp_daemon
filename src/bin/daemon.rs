use clap::Parser;
use daemonize_me::{Daemon, User, Group};
use mcp_daemon::cli::{Cli, Commands};
use mcp_daemon::cli::config::Config;
use mcp_daemon::cli::tui::run_tui;
use std::process;

// Function to be called by the post_fork_child_hook
fn post_fork_child_action(_parent_pid: i32, _child_pid: i32) {
    // This code runs in the child process before privileges are dropped.
    // The original action was println!("Daemon started"), so we'll use a log message.
    // Using tracing/log macros here would be better if the logger is initialized
    // for the child at this stage, but stdout might not be redirected yet.
    // For simplicity, a direct println! can be used, but be aware it might go to the original TTY
    // or be lost if stdout isn't captured yet by the daemon's redirection.
    // Consider logging to a specific file if critical at this stage.
    println!("[Privileged Action Hook - Child Pre-Drop]: Daemon process initialized.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    unsafe {
        std::env::set_var("RUST_LOG", log_level);
    }
    // env_logger::init(); // Will be initialized after potential daemonization

    // Initialize the logger
    // TODO: Configure logger based on CLI args or config file
    // For now, basic init

    // Daemonize the process if not in TUI mode
    if !cli.tui {
        // Configure the daemon
        let daemon_builder = Daemon::new()
            .pid_file("/tmp/mcp_daemon.pid", Some(true)) // Re-enable chmod
            .work_dir("/tmp")
            .user(User::try_from("daemon")?) // Re-enable user
            .group(Group::try_from("daemon")?) // Re-enable group
            .umask(0o000)
            .stdout(std::fs::File::create("/tmp/daemon.out")?)
            .stderr(std::fs::File::create("/tmp/daemon.err")?)
            .setup_post_fork_child_hook(post_fork_child_action) // Added hook
            ;

        // Start the daemonization process
        #[allow(unused_unsafe)] // Allow the lint for this specific, necessary unsafe block
        let start_result = unsafe { daemon_builder.start() };
        match start_result {
            Ok(_) => {
                // Optionally log success or indicate daemon has started if needed
                // For example: log::info!("Daemon successfully started");
            }
            Err(e) => {
                eprintln!("Error starting daemon: {}", e);
                // Depending on requirements, you might want to exit or handle differently
                // std::process::exit(1);
                // return Err(anyhow::anyhow!("Failed to daemonize: {}", e));
            }
        }
    }

    // Initialize env_logger AFTER potential daemonization, so logs go to the right place
    env_logger::init();

    // Load configuration
    let config_path = cli.config.or_else(Config::default_path);
    let config = match config_path {
        Some(path) => {
            if path.exists() {
                match Config::load(&path) {
                    Ok(config) => config,
                    Err(err) => {
                        eprintln!("Error loading configuration: {}", err);
                        Config::default()
                    }
                }
            } else {
                let config = Config::default();
                if let Err(err) = config.save(&path) {
                    eprintln!("Error saving default configuration: {}", err);
                }
                config
            }
        }
        None => {
            eprintln!("No configuration file specified and could not determine default path");
            Config::default()
        }
    };

    // Conditionally launch TUI or proceed with daemon logic
    if cli.tui {
        // Launch the TUI
        // The TUI will block the main thread until it exits
        if let Err(err) = run_tui(config) {
            eprintln!("Error running TUI: {}", err);
            process::exit(1);
        }
    } else {
        // Handle commands
        match cli.command {
            Some(Commands::Start { port }) => {
                println!("Starting MCP Daemon on port {}", port);
                // TODO: Implement daemon startup
                // Add your daemon start logic here
            }
            Some(Commands::Stop) => {
                println!("Stopping MCP Daemon");
                // TODO: Implement daemon shutdown
            }
            Some(Commands::Status) => {
                println!("MCP Daemon status");
                // TODO: Implement status check
            }
            Some(Commands::Add { entity_type, name, url: _ }) => {
                println!("Adding {:?} '{}'", entity_type, name);
                // TODO: Implement entity addition
            }
            Some(Commands::Remove { entity_type, name }) => {
                println!("Removing {:?} '{}'", entity_type, name);
                // TODO: Implement entity removal
            }
            Some(Commands::List { entity_type }) => {
                println!("Listing {:?}s", entity_type);
                // TODO: Implement entity listing
            }
            Some(Commands::Connect { name }) => {
                println!("Connecting to server '{}'", name);
                // TODO: Implement server connection
            }
            Some(Commands::Disconnect { name }) => {
                println!("Disconnecting from server '{}'", name);
                // TODO: Implement server disconnection
            }
            None => {
                // No command specified, run in silent mode
                println!("Running MCP Daemon in silent mode");
                // TODO: Implement silent mode
            }
        }
    }

    Ok(())
}
