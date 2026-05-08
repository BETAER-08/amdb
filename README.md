# amdb: AI Context Generator

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)
![Version](https://img.shields.io/badge/version-0.6.0-blue.svg)
<p align="left">
  <img src="amdb.png" alt="amdb logo" width="60">
</p>

---
## 📄 Related documents

### [korean](readmekr.md)
### [benchmark](benchmark.md)
### [crates.io](https://crates.io/crates/amdb)
---

## ⚡ The Context Problem
AI coding assistants (Cursor, Windsurf, Claude) are powerful, **but they are blind**. They only see the files you open. They lack the deep, structural understanding of your entire codebase that you have.

**`amdb` (Agent Memory Database) solves this.** It scans your local project, builds a vector index of your code, and generates a **single, highly-optimized Markdown context file**. Feed this file to your AI, and watch it understand your project like never before.

---

## 📦 Installation

### Option 1: Manual Download
Prefer to download the file yourself? Go to the Releases Page and download the version for your OS.

### Option 2: Install via Cargo
If you have the Rust toolchain installed:

```bash
cargo install amdb
```
### Option 3: Run via Docker (No Rust required)

You can run `amdb` instantly without setting up a Rust environment by using the official Docker image. This is highly recommended for CI/CD pipelines or non-Rust setups.

```bash
# Pull the latest image from GitHub Container Registry
docker pull ghcr.io/OWNER/amdb:latest

# Initialize the database (mounting the current directory to /app)
docker run --rm -v $(pwd):/app ghcr.io/OWNER/amdb:latest init .

# Generate context
docker run --rm -v $(pwd):/app ghcr.io/OWNER/amdb:latest generate --focus main
```

## 🚀 Quick Start

### 1. Initialize Project
Run this in your project root. `amdb` will scan your code (Rust, Python, JS/TS), extract symbols, and build a vector database in a hidden `.database/` folder.

```bash
amdb init
```

You can also specify a target directory:

```bash
amdb init ./my-project
```

### 2. Generate Context
Create a full project summary. This generates `.amdb/context.md`, which contains a compressed map of your entire codebase.

```bash
amdb generate
```

**🔥 Pro Tip:** Drag and drop `.amdb/context.md` into your AI chat (Cursor/Claude) to give it "God Mode" understanding of your project.

---

## 🧠 Advanced Usage: Focus Mode

For large projects, a full context might be too big. Use **Focus Mode** to generate a summary relevant to a specific feature or bug. `amdb` uses **hybrid search** (exact match first, then vector search) to find the most relevant files.

```bash
# Example: generating context for authentication logic
amdb generate --focus "login authentication jwt"
```

This creates a targeted summary (e.g., in `.amdb/`) containing only the symbols and files relevant to "login authentication jwt".

### 🎯 Depth Control: Expand Context with Call Graph

When using focus mode, you can control how deeply `amdb` explores related files using the **call graph**. The `--depth` flag determines how many levels of function calls to traverse from your initial matches.

```bash
# Depth 0: Only files that exactly match the query
amdb generate --focus "authenticate" --depth 0

# Depth 1 (default): Include files directly called by matched files
amdb generate --focus "authenticate" --depth 1

# Depth 2: Include files 2 levels deep in the call chain
amdb generate --focus "authenticate" --depth 2
```

**How it works:**
1. **Exact Match Priority**: First looks for files/symbols that exactly match your query
2. **Vector Search Fallback**: If no exact matches found, uses semantic similarity search
3. **Call Graph Traversal**: Expands context by following function calls to depth N
4. **Smart Filtering**: Only includes files within similarity threshold (0.25) to keep context relevant

**Example Use Cases:**
- `--depth 0`: When you need only the core implementation (e.g., a single module)
- `--depth 1`: When you need immediate dependencies (default, works for most cases)
- `--depth 2+`: When debugging complex issues that span multiple layers

---

## 🔄 Daemon Mode: Auto-Sync Your Context

Want your AI context to stay fresh automatically? Use **Daemon Mode** to watch your project for changes. When you edit, rename, or delete files, `amdb` instantly updates the database in the background.

```bash
amdb daemon
```

Or specify a directory:

```bash
amdb daemon ./my-project
```

The daemon will:
- ✅ Automatically detect file changes (create, modify, delete, rename)
- ✅ Update the vector database in real-time
- ✅ Keep your context synchronized with your codebase
- ✅ Run silently in the background

**Pro Tip:** Run the daemon in a separate terminal window while you code. Your AI context stays up-to-date without manual `amdb init` runs.

---

## 🛠 Supported Languages

`amdb` uses robust Tree-sitter parsers to fully understand the syntax and structure of:

- **Rust** (`.rs`)
- **Python** (`.py`)
- **JavaScript** (`.js`, `.jsx`, `.mjs`)
- **TypeScript** (`.ts`, `.tsx`)
- **C** (`.c`, `.h`)
- **C++** (`.cpp`, `.hpp`, `.cc`, `.cxx`)
- **C#**(`.cs`)
- **Go** (`.go`)
- **Java** (`.java`)
- **Ruby** (`.rb`)
- **PHP** (.`php`)
- **HTML** (`.html`, `.htm`)
- **CSS** (`.css`)
- **JSON** (`.json`)
- **Bash** (`.sh`, `.bash`)

---

## ⚙️ Configuration

### Custom Configuration (Optional)

You can customize `amdb` behavior by creating an `amdb.toml` file in your project root:

```toml
server_port = 3000

exclude_patterns = [
    "target",
    ".git",
    "node_modules",
    ".amdb",
    ".fastembed_cache",
    "__pycache__",
    "dist",
    "build"
]
```

**Configuration Options:**
- `server_port`: Port for future server features (default: 3000)
- `exclude_patterns`: Directories and patterns to ignore during scanning

### Verbose Mode

Need detailed logs for debugging? Add the `--verbose` (or `-v`) flag to any command:

```bash
amdb init --verbose
amdb generate --verbose
amdb daemon --verbose
```

This outputs detailed debug information about file scanning, parsing, and embedding generation.

---

## 📝 Git Configuration
`amdb` generates local files that should usually be ignored by Git.
Add this to your `.gitignore`:

```text
.database/
.amdb/
```

<p align="center">
  Generated by amdb • The Missing Memory for AI Agents
</p>

Please email us for bug reports or inquiries.
email:try.betaer@gmail.com
