use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::fs;
use console::style;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Start,
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => handle_init()?,
        Commands::Start => println!("{}", style("Starting Daemon...").green()),
        Commands::Status => println!("{}", style("System Normal").blue()),
    }
    Ok(())
}

fn handle_init() -> Result<()> {
    let path = Path::new(".ctx");
    if path.exists() {
        println!("{}", style("Already initialized").yellow());
        return Ok(());
    }

    fs::create_dir(path)?;
    fs::write(path.join("config.toml"), "version = \"0.1\"")?;
    
    println!("{}", style("Initialized .ctx").green());
    Ok(())
}