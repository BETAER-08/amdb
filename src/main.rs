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
    Init,
    Daemon,
    Generate {
        #[arg(short, long)]
        focus: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            tokio::task::spawn_blocking(|| {
                Indexer::scan_project(".")
            }).await??;
        }
        Commands::Daemon => {
            if let Err(e) = FileWatcher::watch(".").await {
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