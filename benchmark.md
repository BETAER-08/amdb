# amdb Benchmark Report — v0.8.0

**Target**: `/home/betaer/RustroverProjects/amdb`
**Method**: Official AMDB Benchmark Suite (`benchmark.py`)
**Codebase Baseline**: 31 files scanned, 19,129 total raw tokens

---

## Summary

amdb converts a full codebase into a compressed, structural, AI-readable context. This report measures how effectively it reduces token volume while preserving retrieval accuracy and dependency structure. All figures are produced by the automated benchmark suite against amdb's own source tree.

| Metric | Score | Meaning |
|--------|-------|---------|
| Precision Targeting | 96.8% | Retrieves the exact file requested |
| Global Efficiency | 97.8% reduction | Token savings vs. full-repo context dump |
| Noise Reduction | 95.1% compression | Interface kept, implementation stripped (complex files) |
| Context Awareness | 100.0% graph inclusion | Dependency graph generated for every run |

---

## Heavyweight Match: Implementation (Raw) vs Interface (amdb)

The five largest source files were selected to evaluate noise reduction under maximum implementation density. Raw tokens represent the full file; amdb tokens represent the extracted interface (symbols, signatures, relationships).

| File | Raw Tokens | amdb Tokens | Compression | Winner |
|------|-----------:|------------:|:-----------:|:------:|
| indexer | 2,601 | 88 | 96.6% | amdb |
| generator | 2,472 | 60 | 97.6% | amdb |
| query | 2,105 | 119 | 94.3% | amdb |
| parser | 1,411 | 71 | 95.0% | amdb |
| vector_store | 1,387 | 108 | 92.2% | amdb |

Across the five heaviest files, amdb reduced 9,976 raw tokens to 446 interface tokens — a 95.5% reduction on the densest portion of the codebase.

---

## Scorecard Detail

### 1. Precision Targeting — 96.8%
Focus queries resolve to the exact file requested with near-perfect accuracy. The slight drop from v0.7.0's 100% reflects a larger codebase (31 files vs. 30, +24% raw tokens) after the v0.8.0 refactors and the new MCP server module widened the retrieval surface.

### 2. Global Efficiency — 97.8% reduction
Against a full-repo context dump of 19,129 tokens, amdb's targeted output consumes roughly 2.2% of the token budget, freeing the remainder for reasoning rather than raw source.

### 3. Noise Reduction — 95.1% compression
On complex files, amdb strips implementation bodies and retains only structural context: symbol names, kinds, signatures, and visibility. This is the interface layer an agent needs to reason about call relationships without reading full function bodies.

### 4. Context Awareness — 100.0% graph inclusion
A Mermaid dependency graph was generated for every run. Graph edges are file-attributed via the post-index `SymbolResolver` (v0.7.0), and as of v0.8.0 the same resolver output backs the `amdb_get_symbol` MCP tool, so callee attribution is consistent between generated reports and live MCP queries.

---

## Interpretation

The core value proposition is token efficiency without loss of structural fidelity. A 97.8% global reduction means an agent can hold the relevant interface of a 31-file project in a fraction of the context a raw dump would require, while the dependency graph preserves the call structure needed for cross-file reasoning.

v0.8.0 makes this context directly consumable by agents: `amdb serve` exposes the same index over MCP (stdio), so the compression measured here applies to every `amdb_get_context` and `amdb_focus` tool call an agent makes — the agent receives the 2.2% interface slice, never the full source dump.

---

## Reproduction

```
python benchmark.py
```

The suite scans the target codebase, measures a raw-token baseline, runs the heavyweight match against the top-5 largest files, and emits the scorecard. Figures in this report correspond to the v0.8.0 release build.

---

## Notes and Limitations

- Compression ratios are highest on implementation-dense files; small or declaration-only files show lower ratios because their raw form is already close to their interface form.
- Token counts are tokenizer-dependent; absolute values will shift with a different tokenizer, though relative reduction remains representative.
- The benchmark measures amdb against its own source tree. Cross-language and larger-repo figures may differ and are tracked separately.
