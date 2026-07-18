# amdb

<p align="left">
  <img src="amdb.png" alt="amdb logo" width="60">
</p>

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)
![Version](https://img.shields.io/badge/version-0.9.0-blue.svg)

**amdb turns your codebase into AI context — entirely on your machine.**

amdb is a zero-runtime, single-binary code context MCP server with combined graph + vector retrieval. No code leaves the machine and no Node/Python runtime is required. Built for air-gapped environments, CI containers, and regulated industries where cloud-based codebase indexing is prohibited.

## Install

```bash
cargo install amdb
```

Or download a static binary for Linux/macOS from the [Releases page](https://github.com/BETAER-08/amdb/releases) — no toolchain required.

## Quickstart

```bash
amdb init .    # index the repo: AST parse + local embeddings, incremental
amdb serve     # expose the index as an MCP server over stdio
```

Done. Prefer a file instead of a server? `amdb generate --focus "auth"` writes a targeted context file to `.amdb/`.

## Connect your editor

**VSCode / Cursor** — add `.vscode/mcp.json` to your project:

```json
{
  "servers": {
    "amdb": {
      "command": "amdb",
      "args": ["serve"]
    }
  }
}
```

**Claude Code**:

```bash
claude mcp add amdb -- amdb serve
```

The server exposes three tools, all reading from the pre-built local index:

| Tool | What it returns |
| :- | :- |
| `amdb_get_context` | Full project overview: files, symbols, and the mermaid dependency graph |
| `amdb_focus` | Context narrowed to a query via name match + semantic vector search, expanded by `depth` dependency hops |
| `amdb_get_symbol` | Every definition of a symbol name as JSON: file, kind, line, signature, callers, and callees (with resolver-accurate files) |

If no index exists the tools respond with an error asking you to run `amdb init` — the server never indexes on its own.

## Demo

Real session, 1.35 seconds end-to-end ([scripts/demo.sh](scripts/demo.sh)):

```console
$ amdb init .
 INFO Files: 33 unchanged, 1 changed, 0 added, 0 removed
 INFO Embedding calls: 0
 INFO Project indexed successfully at .

$ amdb serve
  MCP client calls amdb_get_symbol with {"name": "cosine_similarity"}

cosine_similarity — src/core/vector_store.rs:196
  signature:  fn cosine_similarity(a: &[f32], b: &[f32]) -> f64
  visibility: private
  called by:  search (src/core/vector_store.rs)
  calls:      iter, map, sqrt, sum, zip

Answer came from the local index. No network. No code left the machine.
```

To record the cast on a host with asciinema: `asciinema rec -c "AMDB_BIN=./target/release/amdb ./scripts/demo.sh" demo.cast`, then `agg demo.cast demo.gif`.

## Benchmarks

Measured by [`benchmark.py`](benchmark.py) against amdb's own source tree (31 files, 21,781 raw tokens). Full methodology and caveats in [benchmark.md](benchmark.md).

| Metric | Score | Meaning |
|--------|-------|---------|
| Precision targeting | 100% (28/28 indexed files) | Focus query returns the exact file's own section |
| Global efficiency | 91.5% reduction | Focus output tokens vs. a full-repo dump |
| Noise reduction | 81.6% compression | Interface tokens vs. raw tokens, top-5 largest files |
| Graph presence | 100% (28/28) | Output contains real `-->` dependency edges |

3 of 31 files are module-declaration files with no extractable symbols; they are not in the index and are excluded from the denominator, not silently counted.

## Language support

Symbols and the call graph are extracted for all 16 grammars, but `is_public` and `signature` enrichment is AST-accurate for only three languages. The rest fall back to `is_public = true` and no signature — honest table below, so you know what you get:

| Language | Extensions | Symbols + call graph | `is_public` / `signature` |
|----------|-----------|:---:|:---|
| Rust | `.rs` | ✅ | ✅ AST-accurate |
| Python | `.py` | ✅ | ✅ AST-accurate |
| TypeScript | `.ts`, `.tsx` | ✅ | ✅ AST-accurate |
| JavaScript | `.js`, `.jsx`, `.mjs` | ✅ | fallback (`true` / none) |
| C | `.c`, `.h` | ✅ | fallback (`true` / none) |
| C++ | `.cpp`, `.hpp`, `.cc`, `.cxx` | ✅ | fallback (`true` / none) |
| C# | `.cs` | ✅ | fallback (`true` / none) |
| Go | `.go` | ✅ | fallback (`true` / none) |
| Java | `.java` | ✅ | fallback (`true` / none) |
| Ruby | `.rb` | ✅ | fallback (`true` / none) |
| PHP | `.php` | ✅ | fallback (`true` / none) |
| HTML | `.html`, `.htm` | ✅ | fallback (`true` / none) |
| CSS | `.css` | ✅ | fallback (`true` / none) |
| JSON | `.json` | ✅ | fallback (`true` / none) |
| Bash | `.sh`, `.bash` | ✅ | fallback (`true` / none) |

## How it works

`amdb init` parses every source file with Tree-sitter, extracts symbols and call edges, and embeds each symbol with a local fastembed model — content-hashed, so unchanged files are skipped entirely on re-runs. Everything lands in two SQLite files: a symbol/relationship store and a vector store. Retrieval combines exact name matching, cosine similarity over the vectors, and call-graph expansion, served over MCP stdio or written to a Markdown context file.

## Comparison

Same fixture repo (amdb's own source), same five questions ("where is symbol X defined, and who calls it?"), all numbers actually measured by `benchmark.py`. We did not run competitor indexing tools, so none appear here; the baselines are a raw full-repo dump and a scripted grep-then-read-matched-files agent protocol.

| Strategy | Avg tokens to model | Avg tool calls |
|----------|--------------------:|---------------:|
| Raw full-repo dump | 21,781 | 1 |
| grep + read matched files | 4,161 | 2.4 |
| amdb (`--focus`, depth 1) | 3,972 | 1 |

On a 31-file repo, grep is genuinely competitive on tokens — amdb's edge at this scale is one structured call instead of 2–4, with signatures, visibility, and resolver-accurate caller/callee attribution instead of raw text. The token gap widens with repo size: the dump grows linearly, grep grows with match noise, amdb's focus output grows with the size of the relevant interface.

## More

**Daemon mode** — `amdb daemon` watches the project and incrementally re-indexes on save, keeping the MCP answers fresh.

**Focus depth** — `amdb generate --focus <query> --depth N` expands context N call-graph hops from the matched files (default 1).

**Configuration** — optional `amdb.toml` in the project root:

```toml
db_path = ".database"
ignore_patterns = ["target", ".git", "node_modules", ".amdb", ".fastembed_cache", "__pycache__", ".database"]
```

`AMDB_DB_PATH` overrides `db_path`. Add `.database/` and `.amdb/` to your `.gitignore`.

**Verbose** — `-v` / `--verbose` on any command for debug logs.

## License

MIT. Bug reports and inquiries: try.betaer@gmail.com
