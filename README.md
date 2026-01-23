<div align="center">

# amdb (Agent Memory Database)
### The Open Standard for AI Context Memory

![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-v1.75%2B-orange)
![Version](https://img.shields.io/badge/version-v0.1.0-blue)

</div>

---

## Why amdb?

**amdb** is a local-first code search and indexing engine designed to bridge the gap between AI Agents (like Cursor, Claude) and your codebase's deep context. 

Traditional text search misses the intent; pure vector search misses the structure. **amdb** understands both. It combines semantic understanding with structural graph analysis to provide the "complete picture" of your project to AI models, all while running locally on your machine with zero external API calls.

Whether you are debugging complex authentication flows or refactoring legacy code, amdb ensures your AI agent knows exactly *what* you are looking for and *where* it fits.

---

## Key Features

### 1. 🧠 Hybrid Search (v0.1.0)
The core of amdb is its ability to fuse **Deterministic Call Graphs** with **Semantic Vector Search**.
- **Local Embedding:** Powered by Rust's `fastembed` and `ort`, embeddings are generated locally. No data leaves your machine.
- **Context-Aware:** Searching for "login logic" determines the intent and finds `auth_user` based on code behavior, even if the word "login" is never mentioned.
- **Graph-Enhanced:** It doesn't just find the function; it understands caller/callee relationships to provide the full execution context.

### 2. 🛡️ Security Scanner
amdb treats security as a first-class citizen.
- **Real-time Detection:** Automatically scans for hardcoded secrets (AWS keys, Google API keys, JWT tokens) during indexing.
- **DevSecOps Ready:** Prevents security accidents by warning you before sensitive data enters your context memory.

### 3. ⚡ High Performance
Built with **Rust**, amdb is designed for speed and efficiency.
- **Optimized Storage:** Uses `Bincode` serialization for lightning-fast vector loading (introduced in v0.2.0).
- **Metadata Management:** Leverages SQLite for robust and efficient metadata querying.
- **Zero-Latency:** engineered to serve context to agents in milliseconds.

### 4. 🛠️ Developer Experience
- **Smart Watcher:** Real-time file change detection triggers automatic re-indexing.
- **Polyglot Support:** Native AST parsing and docstring understanding for **Rust**, **Python**, **TypeScript**, and **JavaScript**.
- **Configurable:** Full control via `amdb.toml`.

---

## Installation & Getting Started

### Prerequisites
- Rust v1.75 or higher

### Installation
You can install amdb directly from the source:

```bash
cargo install --path .
```

### Quick Start

1. **Initialize amdb** in your project root:
   ```bash
   amdb init
   ```
   *This analyzes your project structure and downloads the necessary embedding models.*

2. **Start the Daemon**:
   ```bash
   amdb daemon start
   ```
   *This starts the background server on port 3000 (default).*

3. **Search**:
   ```bash
   curl "http://localhost:3000/search?q=auth logic"
   ```

---

## Configuration

Control amdb's behavior using `amdb.toml` in your project root:

```toml
server_port = 3000
exclude_patterns = [
    "target",
    "node_modules",
    "dist",
    ".git",
    "secret_folder"
]
'''
---

## API Usage

amdb exposes a REST API for easy integration with your AI tools or IDE plugins.

### Search Endpoint

```bash
curl -X GET "http://localhost:3000/search?q=database connection"
```

**Response Example:**

```json
[
  {
    "score": 0.89,
    "file": "src/db/connection.rs",
    "id": "src/db/connection.rs::connect_pool",
    "text": "Type: function, Name: connect_pool. Description: Establishes a connection pool..."
  }
]
```

---

## Roadmap

- **v0.2.0:** Improved Semantic Ranking & Caching
- **v0.3.0:** `llms.txt` Generator – Automatically generate context files for LLMs.

---

<div align="center">
  <sub>Built with ❤️ in Rust</sub>
</div>
