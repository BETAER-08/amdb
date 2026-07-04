# Changelog

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

### Changed
- `indexer`'s init path and the daemon's `update_file` shared one `embedding_text(symbol)`
  builder instead of duplicating the format string
- `relationships.caller` now stores a bare symbol name (the `file_path` column already carries
  the file, so the old `file::name` value was redundant)
- `vectors` table gained a `name` column so search boosting compares structured values instead
  of parsing the `id` string

### Investigated, not changed
- `impl_item` signature extraction was suspected to swallow the entire method block; verified
  against the real tree-sitter-rust grammar that `signature_before_body` already cuts at the
  `body` field correctly for `function_item`, `struct_item` and `impl_item` alike (they all use
  the field name `body`). No change made

## [0.6.0] - 2025-05-08

### Fixed
- `is_public` and `signature` fields now correctly saved to and restored from SQLite
- `VectorStore::save()` now executes WAL checkpoint instead of no-op
- `EmbeddingEngine` no longer instantiated twice during `generate --focus`
- File dependency graph is now strictly directional; `--depth` no longer traverses reverse edges
- Mermaid edge output pre-filters by target files before applying the 100-edge cap
- `warnings` table now indexed on `file_path`
- `vectors` table now indexed on `file_path`
- `IndexWorker::update_file` refactored from 4-level nested match to flat `?`-chain
- JWT secret pattern tightened to require minimum 20-char segments, reducing false positives
- Daemon watcher now uses bounded channel (512) with 300ms debounce

### Changed
- `generate --focus` output now includes function signatures when available
- Schema migration guard added for existing databases
