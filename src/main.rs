use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::fs;
use console::style;

mod core;
mod daemon;
mod db;

use core::parser::{CodeParser, SupportedLanguage};
use db::ContextDb;
use daemon::watcher::SystemWatcher;
use daemon::server;

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
    Parse {
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => handle_init()?,
        Commands::Start => handle_start().await?,
        Commands::Status => println!("{}", style("System Normal").blue()),
        Commands::Parse { file } => handle_parse(&file)?,
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

async fn handle_start() -> Result<()> {
    std::thread::spawn(|| {
        if let Err(e) = SystemWatcher::start(".") {
            eprintln!("Watcher failed: {}", e);
        }
    });

    server::start_server().await?;
    
    Ok(())
}

fn handle_parse(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let content = fs::read_to_string(path)?;
    
    let lang = SupportedLanguage::from_path(path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language"))?;
        
    let mut parser = CodeParser::new(lang)?;
    let symbols = parser.parse_symbols(&content)?;
    
    let ctx_path = Path::new(".ctx");
    let mut db = ContextDb::open(ctx_path)?;
    db.save_symbols(file_path, &symbols)?;
    
    println!("{}", style(format!("Parsed & Saved symbols from '{}'", file_path)).bold().green());
    
    println!("---------------------------------------------------");
    db.debug_print_all()?;
    println!("---------------------------------------------------");
     
    Ok(())
}