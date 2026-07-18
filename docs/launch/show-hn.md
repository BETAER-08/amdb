# Show HN draft

## Title

Show HN: Amdb – a single-binary code context MCP server; no code leaves your machine

## Body

I built amdb because I wanted my coding agent to understand my repo without shipping the repo to anyone's cloud.

It's a single Rust binary. `amdb init .` parses your codebase with Tree-sitter, extracts symbols and call edges, embeds each symbol with a local fastembed model (BGE-small, ONNX, runs on CPU), and stores everything in two SQLite files. `amdb serve` then exposes the index over MCP stdio, so VSCode, Cursor, or Claude Code can ask it things like "where is this symbol defined and who calls it" and get back file-accurate answers with signatures and resolver-attributed callers — in milliseconds, fully offline.

What makes it different from the usual RAG-over-code setups:

- Zero runtime. No Node, no Python, no Docker required. One static binary. This matters in CI containers and air-gapped environments where you can't install a toolchain, and in regulated industries where cloud indexing is a compliance non-starter.
- Graph + vector retrieval combined. Exact symbol/file match first, then semantic vector search, then call-graph expansion to N hops. The dependency graph disambiguates same-named symbols across files, and re-resolves attribution live as files change (the daemon watches and re-indexes incrementally — unchanged files are content-hashed and skipped, so a no-op re-index is ~0ms of embedding work).
- Honest scope. Symbols and call graphs for 16 grammars, but visibility/signature enrichment is AST-accurate for Rust, Python, and TypeScript only — the rest fall back to defaults. The README says exactly which.

Numbers, measured against amdb's own source tree with the harness in the repo (I recently found and fixed three biases in my own benchmark script — the corrected, lower numbers are the published ones): a focused context is ~91% smaller than a full-repo dump, and interface extraction compresses the largest files by ~82%. On a small repo, plain grep is honestly competitive on token count — amdb's edge is one structured call instead of several, and answers that carry structure instead of raw text.

Install: `cargo install amdb`, or a static binary from Releases.

Repo: https://github.com/BETAER-08/amdb

Happy to answer questions about the Tree-sitter queries, the symbol resolver, or why I chose SQLite over a vector DB.
