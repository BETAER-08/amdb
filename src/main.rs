use clap::{Parser, Subcommand};
use console::style;
use std::path::Path;
// use tokio::signal; // [Fix] 사용하지 않는 import 제거

mod core;
mod daemon;
mod db;

use crate::core::indexer::Indexer;
use crate::core::parser::{CodeParser, SupportedLanguage};

#[derive(Parser)]
#[command(name = "amdb")]
#[command(about = "Agent Memory Database", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .amdb in the current directory
    Init,
    /// Start the background daemon
    Daemon {
        #[command(subcommand)]
        cmd: DaemonCommands,
    },
    /// Check status
    Status,
    /// Parse a single file (Debug)
    Parse {
        file: String,
    },
    /// Start MCP Server (Stdio)
    Mcp {
        #[command(subcommand)]
        cmd: McpCommands,
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
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            Indexer::scan_project(".")?;
        }
        Commands::Daemon { cmd } => match cmd {
            DaemonCommands::Start => {
                // 1. Watcher 실행 (별도 스레드)
                std::thread::spawn(|| {
                    if let Err(e) = daemon::watcher::SystemWatcher::start(".") {
                        eprintln!("Watcher failed: {}", e);
                    }
                });

                // 2. Server 실행 (메인 스레드 비동기)
                daemon::server::start_server().await?;
            }
        },
        Commands::Status => {
            let path = Path::new(".amdb");
            if path.exists() {
                println!("{}", style("✅ .amdb is initialized.").green());
            } else {
                println!("{}", style("❌ Not initialized. Run 'amdb init' first.").red());
            }
        }
        Commands::Parse { file } => {
            let file_path = &file;
            let path = Path::new(file_path);
            
            if !path.exists() {
                eprintln!("File not found: {}", file_path);
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            if let Some(lang) = SupportedLanguage::from_path(path) {
                let mut parser = CodeParser::new(lang)?;
                
                // 4개 반환값 처리 (warnings 포함)
                let (symbols, graph, _, warnings) = parser.parse(file_path, &content)?;
                
                println!("{}", style(format!("Parsed {}", file_path)).bold());
                println!("Symbols found: {}", symbols.len());
                for s in symbols {
                    println!("  - [{}] {} (line {})", s.kind, s.name, s.line);
                }

                println!("Relationships found: {}", graph.edges.len());
                
                // 경고 출력
                if !warnings.is_empty() {
                    println!("\n{}", style("🚨 Security Warnings:").red().bold());
                    for w in warnings {
                        println!("  - [Line {}] {}: {}", w.line, w.kind, w.message);
                    }
                }

            } else {
                println!("Unsupported language");
            }
        }
        Commands::Mcp { cmd } => match cmd {
            McpCommands::Start => {
                // [Fix] 함수 이름 변경 (start_stdio_server -> run_stdio_server)
                // [Fix] .await 제거 (동기 함수임)
                daemon::mcp::run_stdio_server()?;
            }
        }
    }

    Ok(())
}