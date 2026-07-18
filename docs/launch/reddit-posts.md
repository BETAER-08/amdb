# Reddit drafts

Three separate posts, each written for that subreddit's actual interests. Do not cross-post the same copy.

---

## r/rust

**Title**: amdb — a code-context MCP server in Rust: Tree-sitter + SQLite + local ONNX embeddings in one binary

**Body**:

I've been building amdb, a CLI that indexes a codebase and serves it to AI coding tools over MCP, and I think the implementation details are the interesting part for this sub:

- **Parsing**: `tree-sitter` with per-language queries for 16 grammars. Symbol enrichment (visibility, signatures) is implemented as a `SymbolEnricher` trait with AST-accurate impls for Rust/Python/TypeScript and an explicit fallback for the rest — I'd rather ship an honest trait object than fake signatures with regex.
- **Storage**: two `rusqlite` databases — one for symbols/call edges, one for embedding vectors as BLOBs. I skipped dedicated vector DBs entirely; brute-force cosine over a few thousand vectors with `rayon` par_iter is fast enough and keeps the binary self-contained.
- **Embeddings**: `fastembed` (BGE-small via ONNX Runtime), fully local, no tokens leave the process.
- **Incremental indexing**: xxh3 content hashes per file; unchanged files skip parse and embed entirely. A no-op re-index does zero embedding calls — there's a counter and a test asserting exactly that.
- **File watching**: `notify` + crossbeam channels with a 300ms debounce; the daemon re-resolves cross-file symbol attribution when duplicate names appear or disappear, so the call graph doesn't rot in long sessions.
- **MCP server**: `rmcp` over stdio.

Single binary, `cargo install amdb`. Benchmarks (with a corrected harness — I found and fixed three biases in my own measurement script before publishing) are in the repo. Would appreciate code review, especially of the tree-sitter queries and the resolver.

Repo: https://github.com/BETAER-08/amdb

---

## r/LocalLLaMA

**Title**: Give your local coding model an actual map of your repo — fully offline code indexing over MCP

**Body**:

If you run local models for coding, the context problem is worse than for cloud users: no 200k-context frontier model to dump your repo into, and every wasted token hurts more.

amdb is a single-binary indexer that runs 100% offline. It parses your repo (Tree-sitter), embeds symbols with a local BGE-small ONNX model on CPU (fastembed — no GPU needed, no API calls), and serves the result over MCP stdio to any MCP-capable client. Ask "where is X defined and who calls it" and the model gets a structured answer with file, line, signature, and callers — instead of you pasting files.

Why it fits this sub's workflow:

- Everything local: embedding model, index, retrieval. Nothing phones home; works air-gapped after the one-time model download.
- Token efficiency measured, not vibed: on the project's own source tree, a focused context is ~91% smaller than dumping the repo, and interface extraction strips ~82% of tokens from the biggest files (methodology + corrected harness in the repo — an earlier version of my benchmark script had biases, the published numbers are the fixed ones).
- Small-context friendly: the `--focus` + depth-limited call-graph expansion is designed to produce a few thousand tokens, not fifty thousand.
- No Python/Node runtime — one Rust binary next to your llama.cpp/ollama setup.

Honest caveat: on small repos, grep is competitive on raw token counts — the win is structured single-call answers and staying inside a small context window.

Repo: https://github.com/BETAER-08/amdb

---

## r/selfhosted

**Title**: amdb — self-hosted code context for AI editors: one binary, SQLite, zero cloud

**Body**:

Most "AI understands your codebase" products index your code on their servers. If you self-host your dev stack precisely to avoid that, amdb is the indexing piece that stays home.

- **One static binary** (Rust). No Node, no Python, no containers required — though it runs fine in one. Install via `cargo install amdb` or download from Releases.
- **Your data stays in two SQLite files** inside your repo (`.database/`). Back it up, delete it, grep it — it's yours. No accounts, no telemetry, no outbound connections at runtime.
- **Works with what you already run**: exposes a standard MCP (Model Context Protocol) stdio server, so VSCode, Cursor, and Claude Code — or any self-hosted MCP client — can query your code index locally. Pair it with a local LLM and the entire loop is on your hardware.
- **Set-and-forget**: `amdb daemon` watches the repo and re-indexes incrementally on save (content-hashed, so it only touches changed files).

Setup is two commands: `amdb init .` then `amdb serve`, plus a five-line JSON snippet in your editor config (in the README).

Built for the same reason most of us self-host: the code never leaves the machine. MIT-licensed.

Repo: https://github.com/BETAER-08/amdb
