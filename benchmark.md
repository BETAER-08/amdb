# amdb Benchmark Report — v1.0.0 (corrected harness)

**Target**: amdb's own source tree
**Method**: `benchmark.py` (corrected harness, see "Harness corrections" below)
**Codebase Baseline**: 31 `.rs` files scanned, 21,887 total raw tokens (cl100k_base)
**Indexed**: 28 files — 3 module-declaration files (`src/core/mod.rs`, `src/daemon/mod.rs`, `src/db/mod.rs`) contain no extractable symbols, are not in the index, and are excluded from denominators rather than silently counted.

---

## Harness corrections

The v0.8.0 numbers were produced by a harness with three measurement biases. All three were fixed in v0.9.0; the corrected protocol is what produced every number below, and several figures moved down when it landed.

1. **Interface-extraction regex matched the wrong sections.** The old pattern (`### .*<basename>` with `re.DOTALL`) could span from the first `### ` heading in the document to a later mention of the basename, capturing a fragment of a different file's section. It also could not distinguish the four `mod.rs` files from each other. The corrected harness anchors on the file's full relative path (`^### src/core/indexer.rs$`). **Effect: noise-reduction compression fell from a reported 95.1% to a real 81.7%** (e.g. `indexer` was reported as 88 interface tokens; the real section is 658 tokens).
2. **The graph check only tested for a ` ```mermaid ` fence.** The generator emits that fence unconditionally, even with zero edges, so "100% graph inclusion" was trivially true. The corrected check requires at least one real `-->` edge inside the mermaid block. The score remained 100% (28/28) on this tree — but it is now a meaningful claim.
3. **Query normalization and loose hit-matching.** Queries were derived by replacing underscores with spaces (`"vector store"`), which defeats exact-stem matching and forces vector search; a "hit" was then counted if the basename appeared *anywhere* in the output — including as a dependency of an unrelated match. The corrected harness queries the literal file stem (`vector_store`) and counts a hit only if the output contains the requested file's own `### <path>` section heading, measured at `--depth 0` so dependency expansion cannot mask a retrieval miss.

Additionally, global efficiency fell from a reported 97.8% to 91.5%, reflecting both the corrected protocol and a codebase that has grown since v0.8.0.

---

## Scorecard

| Metric | Score | Protocol |
|--------|-------|----------|
| Precision targeting | 100.0% (28/28) | Query = exact file stem, `--depth 0`; hit = the file's own section heading present |
| Global efficiency | 91.5% reduction | `--depth 1` focus output tokens vs. 21,887-token full dump, averaged over 28 queries |
| Noise reduction | 81.7% compression | Interface section tokens vs. raw file tokens, top-5 largest files |
| Graph presence | 100.0% (28/28) | At least one real `-->` edge inside the mermaid block at `--depth 1` |

---

## Heavyweight Match: Implementation (Raw) vs Interface (amdb)

Top-5 largest files, interface section extracted by exact path anchor:

| File | Raw Tokens | amdb Tokens | Compression |
|------|-----------:|------------:|:-----------:|
| indexer | 4,630 | 658 | 85.8% |
| query | 2,659 | 713 | 73.2% |
| generator | 2,472 | 484 | 80.4% |
| mcp | 2,170 | 372 | 82.9% |
| parser | 1,411 | 191 | 86.5% |

Across the five heaviest files: 13,342 raw tokens → 2,418 interface tokens (81.9% reduction).

---

## Comparison: amdb vs raw dump vs grep-based agentic search

Fixed task set: five questions of the form *"where is symbol X defined, and which functions call it?"* on the same fixture repo. Baseline protocols:

- **Raw full-repo dump**: feed the entire `src/` tree; 1 call.
- **grep agentic**: one `grep -rn <symbol> src` call, then read every matched file in full — the minimal loop a grep-only agent performs. Tokens = grep output + full text of matched files.
- **amdb**: one `amdb generate --focus <symbol>` call (default depth 1). Tokens = the focus output.

We did **not** run competitor indexing tools (Sourcegraph Cody, Aider repo-map, etc.), so no numbers for them appear here. These are the only two baselines we actually executed.

| Question (symbol) | amdb tokens | calls | raw dump tokens | calls | grep tokens | calls |
|---|---:|---:|---:|---:|---:|---:|
| cosine_similarity | 3,859 | 1 | 21,887 | 1 | 1,434 | 2 |
| reresolve_delta | 4,434 | 1 | 21,887 | 1 | 4,858 | 2 |
| normalize_path | 2,604 | 1 | 21,887 | 1 | 5,219 | 3 |
| focus_filename | 4,530 | 1 | 21,887 | 1 | 4,701 | 3 |
| content_hash | 4,432 | 1 | 21,887 | 1 | 4,687 | 2 |
| **average** | **3,972** | **1** | **21,887** | **1** | **4,180** | **2.4** |

Honest reading: on a 31-file repo, grep-then-read is competitive with amdb on tokens (4,180 vs 3,972) and even wins when the symbol lives in one small file (`cosine_similarity`). amdb's advantages at this scale are one structured call instead of 2–3, and answers that carry signatures, visibility, and resolver-attributed caller/callee files instead of raw text the model must re-parse. The raw dump costs ~5.5× more tokens per question and grows linearly with repo size.

---

## Reproduction

```
pip install tiktoken
AMDB_BIN=./target/release/amdb python3 benchmark.py
```

The harness always runs `amdb init` first (incremental — unchanged files are skipped), reads the indexed file list from `.database/context.db`, runs each file query at depth 0 and depth 1, then runs the comparison protocols.

---

## Notes and Limitations

- All figures measured on amdb's own source tree; a self-benchmark is a convenience fixture, not a claim about your repo. Larger and cross-language repos will differ.
- Token counts use `cl100k_base`; absolute values shift with tokenizer choice, relative reductions less so.
- Compression is highest on implementation-dense files; declaration-heavy files compress less because their raw form already resembles their interface.
- Precision is measured with exact-stem queries — it is a retrieval-plumbing test, not a semantic-search quality test. Vector-search quality for fuzzy queries is not covered by this harness.
