# awesome-mcp-servers PR draft

## One-line entry

Place under a code/developer-tools section, keeping the list's alphabetical order and emoji conventions of the target section:

```markdown
- [BETAER-08/amdb](https://github.com/BETAER-08/amdb) 🦀 🏠 - Single-binary local code context server. Tree-sitter AST + call graph + local vector embeddings over SQLite; symbol lookup, focused context, and dependency-graph tools. No runtime dependencies, fully offline.
```

(`🦀` = Rust, `🏠` = local/self-hosted service, per the repo's legend; adjust emoji to the target list's current legend before opening the PR.)

## PR title

Add amdb — local single-binary code context server

## PR description

This adds amdb, an MCP server (stdio) that turns a codebase into queryable AI context entirely on the local machine.

- **What it does**: `amdb init` indexes a repo (Tree-sitter symbol extraction, call-graph edges, local fastembed embeddings, SQLite storage). `amdb serve` exposes three tools: `amdb_get_context` (project overview + mermaid dependency graph), `amdb_focus` (query-narrowed context via exact match + vector search + graph expansion), and `amdb_get_symbol` (definitions with file, line, signature, callers, callees).
- **Why it belongs here**: it is a working, tested MCP server over stdio (integration tests exercise the JSON-RPC handshake and all three tools), installable with one command (`cargo install amdb`) or a static binary, and requires no Node/Python runtime — useful for air-gapped and CI environments.
- **License**: MIT.
- **Language**: Rust.

Checked the contribution guidelines: entry is one line, alphabetically placed, description written to match the list's style.
