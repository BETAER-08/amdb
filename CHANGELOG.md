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

### Added
- `SymbolEnricher` trait (`core::languages`) with per-language `is_public`/`signature`
  implementations for Rust, Python and TypeScript; other supported languages fall back to
  `(true, None)`, documented in the README language table
- `normalize_path(root, path)` helper for consistent file identity across `init` and the daemon
- SQLite schema versioning via `PRAGMA user_version`: a pre-0.7 database is detected and its
  stale-format `relationships` rows are purged instead of crashing on read

### Changed
- `indexer`'s init path and the daemon's `update_file` shared one `embedding_text(symbol)`
  builder instead of duplicating the format string
- `relationships.caller` now stores a bare symbol name (the `file_path` column already carries
  the file, so the old `file::name` value was redundant)
- `vectors` table gained a `name` column so search boosting compares structured values instead
  of parsing the `id` string
- Five integration tests rewritten from substring "contains" checks to exact-value assertions
  (line number, is_public, signature contents, an actual mermaid edge arrow, and a
  `std::collections::HashMap`-in-a-call regression case for the identity refactor above)

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
