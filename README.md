<div align="center">

# .amdb

**The Context Protocol**
<br>
_The Open Standard for AI Context Memory_

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

</div>

---

## 📐 The Missing Pillar

**.amdb** (pronounced "dot-context") is a local daemon that turns any project into a self-explaining entity. It ensures that every AI tool—whether it's GitHub Copilot, Cursor, or a local LLM—shares a single, unified understanding of your codebase.

## 🚀 Why .amdb?

**The Problem: Siloed Intelligence**
Every new AI tool you use re-indexes your code from scratch. They are strangers to your project, guessing context based on open files or naive text chunking. They don't talk to each other, and they forget everything when you close the session.

**The Solution: A Shared Protocol**
`.amdb` runs locally, watching your file system. It proactively parses code, understands dependencies, and maintains a high-fidelity context map in a standardized `.amdb/` directory.
- **Write Once:** Your context is calculated once.
- **Read Everywhere:** Exposed via **MCP (Model Context Protocol)** to any supported editor or agent.

## 🏗️ Architecture

The system is designed as a modular, high-performance daemon built in Rust.

```mermaid
graph LR
    FS[File System] -->|Notify| Watcher
    Watcher -->|Diff| Parser[Tree-sitter Parser]
    Parser -->|AST| Engine
    Engine -->|Embeddings| DB[(.amdb/store.db)]
    DB -->|Query| MCPServer[MCP Server]
    MCPServer -->|JSON-RPC| Client[Cursor / Claude / Copilot]
```

### Folder Structure
The `.amdb` folder acts as the brain of your repository:
- `config.toml`: Protocol settings (ignore patterns, language policies).
- `store.db`: SQLite database storing semantic relationships and symbols.
- `vector/`: Local vector store for semantic search.

## ⚡ Getting Started

### Installation
(Assuming crate publication)
```bash
cargo install amdb
```

### Initialization
Turn your current directory into a Context-Aware project:
```bash
amdb init
```
This creates the `.amdb` skeleton and begins the initial indexing process.

### Usage
Start the background daemon to keep context in sync:
```bash
amdb daemon start
```

Check the status of the context index:
```bash
amdb status
```

## 🔌 Integration

### Cursor / VS Code
`.amdb` generates a dynamic `.cursorrules` file or exposes a local server that Cursor allows you to hook into, providing "God-mode" context awarness without uploading your code to the cloud.

### Claude Desktop
Configure your `claude_desktop_config.json` to use the locally running .amdb MCP server:

```json
{
  "mcpServers": {
    "amdb": {
      "command": "amdb",
      "args": ["mcp", "start"]
    }
  }
}
```

## 🗺️ Roadmap

- [x] **Phase 1: Local Context Map** (Complete)
    - File watching, Tree-sitter parsing, and SQLite storage.
- [ ] **Phase 2: Semantic Vector Sync** (In Progress)
    - Local embedding generation and RAG interface.
- [ ] **Phase 3: Global Standard**
    - Native integration plugins for JetBrains and VS Code.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
