use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::fs;
use console::style;

#[derive(Parser)]
#[command(name = "ctx")]
#[command(author = "Builder <builder@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Manage AI context for your project", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Start {
        #[arg(short, long)]
        debug: bool,
    },
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            handle_init()?;
        }
        Commands::Start { debug } => {
            let mode = if *debug { "Debug" } else { "Production" };
            println!(
                "{} Starting .CTX Daemon in {} mode...", 
                style("[SYSTEM]").bold().green(), 
                style(mode).yellow()
            );
        }
        Commands::Status => {
            println!(
                "{} Project context is {}", 
                style("[INFO]").bold().blue(), 
                style("NOT SYNCED").red()
            );
        }
    }

    Ok(())
}

fn handle_init() -> Result<()> {
    let path = Path::new(".ctx");
    
    if path.exists() {
        println!("{} .ctx directory already exists.", style("[SKIP]").yellow());
        return Ok(());
    }

    fs::create_dir(path)?;
    fs::write(path.join("config.toml"), "# .CTX Configuration\nversion = \"0.1\"")?;
    
    println!(
        "{} Initialized empty context repository in .ctx/", 
        style("[SUCCESS]").bold().green()
    );
    return Ok(());
}