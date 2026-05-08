# Changelog

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
