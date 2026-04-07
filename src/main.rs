use clap::{Parser, Subcommand};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod core;
mod daemon;
mod db;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match cli.command {
        Commands::Init { path } => {
            info!("Initializing amdb in: {}", path);
            let res = tokio::task::spawn_blocking(move || Indexer::scan_project(&path)).await?;

            if let Err(e) = res {
                error!("Init failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Daemon { path } => {
            info!("Starting daemon watcher on: {}", path);
            if let Err(e) = FileWatcher::watch(&path).await {
                error!("Watcher error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Generate { focus, depth } => {
            if let Err(e) = ContextGenerator::generate(focus, depth).await {
                error!("Generation error: {}", e);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}