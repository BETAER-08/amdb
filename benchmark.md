# amdb Benchmark Report — v0.7.0

**Target**: `/home/betaer/RustroverProjects/amdb`
**Method**: Official AMDB Benchmark Suite (`benchmark.py`)
**Codebase Baseline**: 30 files scanned, 15,409 total raw tokens

---

## Summary

amdb converts a full codebase into a compressed, structural, AI-readable context. This report measures how effectively it reduces token volume while preserving retrieval accuracy and dependency structure. All figures are produced by the automated benchmark suite against amdb's own source tree.

| Metric | Score | Meaning |
|--------|-------|---------|
| Precision Targeting | 100.0% | Retrieves the exact file requested |
| Global Efficiency | 97.5% reduction | Token savings vs. full-repo context dump |
| Noise Reduction | 94.5% compression | Interface kept, implementation stripped (complex files) |
| Context Awareness | 100.0% graph inclusion | Dependency graph generated for every run |

---

## Heavyweight Match: Implementation (Raw) vs Interface (amdb)

The five largest source files were selected to evaluate noise reduction under maximum implementation density. Raw tokens represent the full file; amdb tokens represent the extracted interface (symbols, signatures, relationships).

| File | Raw Tokens | amdb Tokens | Compression | Winner |
|------|-----------:|------------:|:-----------:|:------:|
| generator | 2,146 | 60 | 97.2% | amdb |
| indexer | 1,988 | 88 | 95.6% | amdb |
| query | 1,590 | 119 | 92.5% | amdb |
| parser | 1,411 | 71 | 95.0% | amdb |
| vector_store | 1,387 | 108 | 92.2% | amdb |

Across the five heaviest files, amdb reduced 8,522 raw tokens to 446 interface tokens — a 94.8% reduction on the densest portion of the codebase.

---

## Scorecard Detail

### 1. Precision Targeting — 100.0%
Every focus query resolved to the exact file requested. No false-positive files were included in the targeted context.

### 2. Global Efficiency — 97.5% reduction
Against a full-repo context dump of 15,409 tokens, amdb's targeted output consumes roughly 2.5% of the token budget, freeing the remainder for reasoning rather than raw source.

### 3. Noise Reduction — 94.5% compression
On complex files, amdb strips implementation bodies and retains only structural context: symbol names, kinds, signatures, and visibility. This is the interface layer an agent needs to reason about call relationships without reading full function bodies.

### 4. Context Awareness — 100.0% graph inclusion
A Mermaid dependency graph was generated for every run. As of v0.7.0, graph edges are file-attributed via the post-index `SymbolResolver`, so same-named symbols in different files render as distinct nodes rather than collapsing into one.

---

## Interpretation

The core value proposition is token efficiency without loss of structural fidelity. A 97.5% global reduction means an agent can hold the relevant interface of a 30-file project in a fraction of the context a raw dump would require, while the dependency graph preserves the call structure needed for cross-file reasoning.

The v0.7.0 symbol-resolution work directly affects the Context Awareness metric: prior to this release, duplicate-named functions across files collapsed into single graph nodes, which would have overstated connectivity. File-attributed edges make the 100% graph-inclusion figure structurally accurate, not merely present.

---

## Reproduction

```
python benchmark.py
```

The suite scans the target codebase, measures a raw-token baseline, runs the heavyweight match against the top-5 largest files, and emits the scorecard. Figures in this report correspond to the v0.7.0 release build.

---

## Notes and Limitations

- Compression ratios are highest on implementation-dense files; small or declaration-only files show lower ratios because their raw form is already close to their interface form.
- Token counts are tokenizer-dependent; absolute values will shift with a different tokenizer, though relative reduction remains representative.
- The benchmark measures amdb against its own source tree. Cross-language and larger-repo figures may differ and are tracked separately.
