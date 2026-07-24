# Changelog

## [Unreleased]

### Added
- `.dockerignore`, and a serving Docker entrypoint: the container image now starts the
  MCP stdio server by default (`ENTRYPOINT ["amdb", "serve"]`, working directory
  `/workspace`), matching what Glama's automated server check expects; run other
  subcommands via `--entrypoint amdb`
- `scripts/glama-check.sh` reproducing Glama's container check locally

## [1.0.0] 

Stability release. No new features; the public contract is now frozen and covered by
contract tests (`tests/contract_test.rs`). Breaking changes to anything listed under
"Frozen" now require a 2.0 release. See the README "Stability" section for the full
covered/not-covered list.

### Frozen
- CLI surface: `init`, `daemon`, `generate`, `serve`; `--focus`/`-f`, `--depth`/`-d`,
  `--verbose`/`-v`; optional path argument on `init`/`daemon`; exit codes 0/1
- MCP tools: exactly `amdb_get_context`, `amdb_focus`, `amdb_get_symbol` with their
  current input parameters; `amdb_get_symbol` response fields `file`, `name`, `kind`,
  `line`, `signature`, `is_public`, `callers[] {name, file}`,
  `callees[] {name, file, resolution}` (additive evolution only)
- Config: `amdb.toml` keys `db_path` and `ignore_patterns`; `AMDB_DB_PATH` override
- Database upgrades: automatic migration via `PRAGMA user_version` from any 0.6+ index;
  deleting `.database/` is never a required upgrade step
- Generated Markdown anchors: the `### <relative/path>` per-file heading and the single
  fenced mermaid block with `-->` edges; all other layout details remain unstable

### Added
- `callees[].resolution` in `amdb_get_symbol` responses, exposing how each callee's file
  was attributed by the current resolver: `same-file`, `global-unique`, or `unresolved`.
  Added now so that future language-specific resolution strategies extend the value set
  additively instead of changing the response shape
- Contract tests guarding the CLI surface, MCP tool names, the `amdb_get_symbol`
  response schema, the documented config keys, and the migration chain from each prior
  `PRAGMA user_version` (0, 1, 2)

### Fixed
- Legacy-schema migration purged `symbols` and `relationships` but left `file_hashes`
  intact, so the next `amdb init` skipped every unchanged file and left the index empty.
  The migration now purges `file_hashes` too, forcing the next `init` to rebuild fully.
  No released version could hit this (hash-based indexing shipped after the last schema
  bump), but the chain would have broken at the next `user_version` increase

### Benchmarks
- Re-measured on the v1.0.0 source with the corrected harness introduced in v0.9.0:
  precision targeting 100% (28/28), global efficiency 91.5%, noise reduction 81.7%
  (was 81.6% at v0.9.0), graph presence 100% (28/28). Baseline grew to 21,887 raw
  tokens; grep-baseline average moved to 4,180 tokens. No headline figure moved
  materially in the audit — the corrections themselves landed in v0.9.0, where noise
  reduction fell from a reported 95.1% to 81.6% and global efficiency from 97.8% to
  91.5%

## [0.9.0] - 2026-07-18

### Added
- Incremental indexing via xxh3 content hashes: unchanged files skip parsing and
  embedding entirely; `init` reports unchanged/changed/added/removed counts
- Daemon re-resolution of cross-file callee attribution when duplicate symbol names
  appear or disappear

### Fixed
- Three measurement biases in `benchmark.py` (interface-extraction regex, fence-only
  graph check, query normalization); published figures corrected downward accordingly

## [0.8.0] - 2026-07-09

### Added
- MCP server over stdio (`amdb serve`) exposing `amdb_get_context`, `amdb_focus`, and
  `amdb_get_symbol`
- Non-zero exit code on unrecoverable errors

## [0.7.0] - 2026-07-05

### Fixed
- `CodeSymbol.line` was populated from the tree-sitter query's pattern index, not a source
  line; it's now the definition capture's actual `start_position().row + 1`
- `is_public` and `signature` were always `false`/empty because no query emitted `@pub`/`@sig`
  captures; they're now derived by direct AST traversal (see Added) for Rust, Python and
  TypeScript
- The shared JavaScript/TypeScript query captured class names as `(identifier)`, which is
  invalid against the TypeScript grammar (class names are `type_identifier` there); every
  `.ts`/`.tsx` file was silently producing zero symbols. Split into `QUERY_JS`/`QUERY_TS`
- Symbol/edge identity was encoded as `format!("{}::{}", file, name)` strings and recovered via
  `split("::")`, which corrupts on names containing `::` and is fragile on Windows paths;
  replaced with a structured `SymbolRef { file, name }` used internally end-to-end
- File paths are now normalized (relative to project root, forward slashes) at ingestion via a
  single shared helper used by both `init` and the daemon, instead of storing whatever format
  each caller happened to produce
- `vector_store::search`'s graph-boosting code path was dead: `resolve_focus_targets` always
  passed `graph: None`. `generate` now builds a project-wide dependency graph once and threads
  it through, so boosting by call relationships actually runs
- Mermaid node IDs were asymmetric: a caller's ID was sanitized from its full `file::name`
  form while the same symbol's ID as a callee was sanitized from the bare name alone, so a
  symbol appearing on both sides of a call chain got two different node IDs and the rendered
  graph looked disconnected even where a real edge existed
- `schema::init`'s `ALTER TABLE ADD COLUMN` migration caught every `SqliteFailure`, not just
  "duplicate column name" — any other structural failure at that point would have been
  silently swallowed. Narrowed to `is_duplicate_column_error`, which matches on the specific
  message text
- The migration purged only the `relationships` table; a legacy `symbols` row (stale `line`,
  always-false `is_public`) survived until its file was re-touched. The migration now also
  purges `symbols`, consistent with how `relationships` was already handled
- Two functions sharing a name in different files collapsed into a single mermaid node, and
  `relationships.callee` carried no file attribution at all. A post-index `SymbolResolver`
  now attributes each call edge to a `callee_file` (same-file match, else global-unique
  match, else left unresolved), and the mermaid renderer builds both node IDs from
  `(file, name)` via one `sanitize_node_id` rule, so same-named symbols in different files
  render as distinct nodes

### Added
- `SymbolEnricher` trait (`core::languages`) with per-language `is_public`/`signature`
  implementations for Rust, Python and TypeScript; other supported languages fall back to
  `(true, None)`, documented in the README language table
- `normalize_path(root, path)` helper for consistent file identity across `init` and the daemon
- SQLite schema versioning via `PRAGMA user_version`: a pre-0.7 database is detected and its
  stale-format `relationships` and `symbols` rows are purged instead of crashing on read
- `SymbolResolver` (`core::symbol`) resolving a callee name to its defining file when
  unambiguous; `relationships` gained a nullable `callee_file` column populated by a
  post-index resolution pass in `init`'s full scan. The daemon's incremental `update_file`
  applies only the cheap same-file rule (it doesn't have the whole project's symbol table
  available); cross-file/global-unique attribution for daemon-touched files is deferred until
  the next full `init`, rather than guessed
- Strict regression tests covering: exact line number, is_public true/false, signature
  contents, a real mermaid edge arrow, a `std::collections::HashMap`-in-a-call regression,
  mermaid node-ID symmetry, callee-resolution ambiguity pinning, a migration-error
  classifier (duplicate vs. genuinely different `SqliteFailure`), a legacy-DB purge proof,
  and the symbol resolver (distinct per-file nodes for a duplicate name, `callee_file`
  persisted for a globally unique symbol, and an ambiguous duplicate left unresolved or
  split rather than guessed)
