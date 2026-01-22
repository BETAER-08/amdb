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
use daemon::mcp;
use core::indexer::Indexer; 

#[derive(Parser)]
#[command(name = "amdb", version = "0.1", about = "The Context Protocol: Open Standard for AI Context Memory")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    
    Status,

    Daemon {
        #[command(subcommand)]
        cmd: DaemonCommands,
    },

    Mcp {
        #[command(subcommand)]
        cmd: McpCommands,
    },

    Parse {
        file: String,
    },
}

#[derive(Subcommand)]
enum DaemonCommands {
    Start,
}

#[derive(Subcommand)]
enum McpCommands {
    Start,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => handle_init()?,
        Commands::Status => handle_status()?,

        Commands::Daemon { cmd } => match cmd {
            DaemonCommands::Start => {
                println!("{}", style("Starting .amdb Daemon (HTTP Server)...").green().bold());
                println!("Supported Integration: Custom Agents, Postman, Curl");
                handle_daemon_start().await?
            }
        },

        Commands::Mcp { cmd } => match cmd {
            McpCommands::Start => {
                eprintln!("Starting .amdb MCP Server (Stdio Mode)...");
                mcp::run_stdio_server()?;
            }
        },

        Commands::Parse { file } => handle_parse(&file)?,
    }
    Ok(())
}

fn handle_init() -> Result<()> {
    let path = Path::new(".amdb");
    if path.exists() {
        println!("{}", style("Already initialized").yellow());
        Indexer::scan_project(".")?;
        return Ok(());
    }

    fs::create_dir(path)?;

    fs::create_dir(path.join("vector")).unwrap_or_default();

    fs::write(path.join("config.toml"), "version = \"0.1\"\n[ignore]\npatterns = [\"target\", \".git\"]")?;
    
    println!("{}", style("Initialized .amdb project").green());
    println!("  - Created .amdb/ directory");
    println!("  - Created .amdb/store.db (via SQLite)");
    println!("  - Created .amdb/vector/ (for Semantic Search)");

    Indexer::scan_project(".")?;
    
    Ok(())
}

fn handle_status() -> Result<()> {
    let path = Path::new(".amdb");
    if path.exists() {
        println!("{}", style("Status: Active").green().bold());

        let db_path = path.join("store.db");
        if db_path.exists() {
            println!("Context Store: {}", style("Connected").cyan());
            println!("DB Location: .amdb/store.db");
        } else {
            println!("Context Store: {}", style("Empty (Run daemon to index)").yellow());
        }
        
        println!("Protocol Version: 0.1");
    } else {
        println!("{}", style("Status: Not Initialized").red());
        println!("Run 'amdb init' to start.");
    }
    Ok(())
}

async fn handle_daemon_start() -> Result<()> {
    std::thread::spawn(|| {
        let _ = Indexer::scan_project(".");
    });

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
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found"));
    }
    
    let content = fs::read_to_string(path)?;
    let lang = SupportedLanguage::from_path(path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language"))?;
        
    let mut parser = CodeParser::new(lang)?;
    let (symbols, graph, _) = parser.parse(file_path, &content)?;
    
    let amdb_path = Path::new(".amdb");
    if !amdb_path.exists() {
        return Err(anyhow::anyhow!("Project not initialized. Run 'amdb init' first."));
    }

    let mut db = ContextDb::open(amdb_path)?;
    db.save_symbols(file_path, &symbols)?;
    db.save_relationships(file_path, &graph)?;
    
    println!("{}", style(format!("Parsed & Saved symbols from '{}'", file_path)).bold().green());
    graph.debug_print();
    
    Ok(())
}