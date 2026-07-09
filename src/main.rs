use anyhow::Context;
use clap::{Parser, Subcommand};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod core;
mod daemon;
mod db;
mod mcp;

use crate::core::generator::ContextGenerator;
use crate::core::indexer::Indexer;
use crate::daemon::watcher::FileWatcher;

#[derive(Parser)]
#[command(name = "amdb")]
#[command(about = "Agent Memory Database", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
    Daemon {
        #[arg(default_value = ".")]
        path: String,
    },
    Generate {
        #[arg(short, long)]
        focus: Option<String>,

        #[arg(short, long, default_value_t = 1)]
        depth: u8,
    },
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    if matches!(cli.command, Commands::Serve) {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(log_level)
            .with_target(false)
            .without_time()
            .with_writer(std::io::stderr)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    } else {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(log_level)
            .with_target(false)
            .without_time()
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }

    let result = match cli.command {
        Commands::Init { path } => {
            info!("Initializing amdb in: {}", path);
            tokio::task::spawn_blocking(move || Indexer::scan_project(&path))
                .await?
                .context("Init failed")
        }
        Commands::Daemon { path } => {
            info!("Starting daemon watcher on: {}", path);
            FileWatcher::watch(&path).await.context("Watcher error")
        }
        Commands::Generate { focus, depth } => ContextGenerator::generate(focus, depth)
            .await
            .context("Generation error"),
        Commands::Serve => {
            info!("Starting amdb MCP server on stdio");
            mcp::serve_stdio().await.context("Serve error")
        }
    };

    if let Err(e) = result {
        error!("{:#}", e);
        std::process::exit(1);
    }
    Ok(())
}
