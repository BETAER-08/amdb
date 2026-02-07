use clap::{Parser, Subcommand};

mod core;
mod daemon;
mod db;

use crate::core::indexer::Indexer;
use crate::daemon::watcher::FileWatcher;
use crate::core::generator::ContextGenerator;

#[derive(Parser)]
#[command(name = "amdb")]
#[command(about = "Agent Memory Database", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            tokio::task::spawn_blocking(move || {
                Indexer::scan_project(&path)
            }).await??;
        }
        Commands::Daemon { path } => {
            if let Err(e) = FileWatcher::watch(&path).await {
                eprintln!("Watcher error: {}", e);
            }
        }
        Commands::Generate { focus } => {
            if let Err(e) = ContextGenerator::generate(focus).await {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}