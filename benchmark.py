import glob
import heapq
import os
import re
import sqlite3
import subprocess
import sys

try:
    import tiktoken
except ImportError:
    print("❌ Error: 'tiktoken' library is required.")
    print("👉 Run: pip install tiktoken")
    sys.exit(1)

ENCODER = tiktoken.get_encoding("cl100k_base")
AMDB = os.environ.get("AMDB_BIN", "amdb")

COMPARISON_QUESTIONS = [
    "cosine_similarity",
    "reresolve_delta",
    "normalize_path",
    "focus_filename",
    "content_hash",
]


def run_command(cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=False)


def count_tokens(text):
    return len(ENCODER.encode(text))


def scan_project_files(root_dir="src"):
    file_list = []
    total_tokens = 0

    for root, _, files in os.walk(root_dir):
        for file in sorted(files):
            if not file.endswith(".rs"):
                continue
            full_path = os.path.join(root, file)
            rel_path = full_path.replace(os.sep, "/")

            with open(full_path, "r", encoding="utf-8") as f:
                content = f.read()
            raw_len = count_tokens(content)
            total_tokens += raw_len

            file_list.append(
                {
                    "query": file[: -len(".rs")],
                    "rel_path": rel_path,
                    "tokens": raw_len,
                }
            )

    return file_list, total_tokens


def indexed_files(db_path=".database/context.db"):
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute("SELECT DISTINCT file_path FROM symbols").fetchall()
    finally:
        conn.close()
    return {row[0] for row in rows}


def generate_focus(query, depth):
    for f in glob.glob(".amdb/*.md"):
        os.remove(f)

    run_command([AMDB, "generate", "--focus", query, "--depth", str(depth)])

    generated = glob.glob(".amdb/*.md")
    if not generated:
        return None
    with open(generated[0], "r", encoding="utf-8") as f:
        return f.read()


def has_file_section(content, rel_path):
    return re.search(rf"^### {re.escape(rel_path)}$", content, re.MULTILINE) is not None


def extract_interface_section(content, rel_path):
    pattern = re.compile(
        rf"^### {re.escape(rel_path)}\n(.*?)(?=^#|\Z)",
        re.MULTILINE | re.DOTALL,
    )
    match = pattern.search(content)
    if not match:
        return ""
    return match.group(1).strip()


def has_graph_edges(content):
    match = re.search(r"```mermaid\n(.*?)```", content, re.DOTALL)
    if not match:
        return False
    return "-->" in match.group(1)


def grep_agentic_baseline(query, file_tokens_by_path):
    tool_calls = 1
    grep = run_command(["grep", "-rn", query, "src"])
    tokens = count_tokens(grep.stdout)

    matched_files = []
    for line in grep.stdout.splitlines():
        path = line.split(":", 1)[0].replace(os.sep, "/")
        if path not in matched_files:
            matched_files.append(path)

    for path in matched_files:
        tool_calls += 1
        if path in file_tokens_by_path:
            tokens += file_tokens_by_path[path]
        elif os.path.exists(path):
            with open(path, "r", encoding="utf-8") as f:
                tokens += count_tokens(f.read())

    return tokens, tool_calls, len(matched_files)


def run_comparison(file_list, total_project_tokens):
    file_tokens_by_path = {f["rel_path"]: f["tokens"] for f in file_list}
    rows = []

    for query in COMPARISON_QUESTIONS:
        amdb_output = generate_focus(query, 1)
        amdb_tokens = count_tokens(amdb_output) if amdb_output else 0

        grep_tokens, grep_calls, grep_files = grep_agentic_baseline(
            query, file_tokens_by_path
        )

        rows.append(
            {
                "query": query,
                "amdb_tokens": amdb_tokens,
                "amdb_calls": 1,
                "raw_tokens": total_project_tokens,
                "raw_calls": 1,
                "grep_tokens": grep_tokens,
                "grep_calls": grep_calls,
                "grep_files": grep_files,
            }
        )

    return rows


def run_benchmark():
    print("\n🚀 Starting AMDB Benchmark Suite (corrected harness)")
    print(f"   Target: {os.getcwd()}")
    print(f"   Binary: {AMDB}")
    print("-" * 60)

    print("⚙️ Building index (incremental)...", end=" ", flush=True)
    run_command([AMDB, "init"])
    print("Done.")

    print("📊 Measuring Codebase Baseline...", end=" ", flush=True)
    file_list, total_project_tokens = scan_project_files("src")
    indexed = indexed_files()
    print("Done.")
    print(f"   - Files Scanned: {len(file_list)}")
    print(f"   - Total Raw Tokens: {total_project_tokens:,} (full codebase size)")

    benchmarked = [f for f in file_list if f["rel_path"] in indexed]
    unindexed = [f for f in file_list if f["rel_path"] not in indexed]
    print(f"   - Indexed Files: {len(benchmarked)}")
    if unindexed:
        print(
            f"   - Excluded (no extractable symbols, not retrievable): {len(unindexed)}"
        )
        for f in unindexed:
            print(f"       {f['rel_path']}")

    results = {
        "retrieval_hits": 0,
        "graph_hits": 0,
        "global_reduction_sum": 0,
        "file_stats": [],
    }

    print("\n⚔️ Running Retrieval + Compression Tests...")
    print("   Query protocol: query = exact file stem; hit requires the file's own")
    print("   '### <path>' section heading in the depth-0 output.")

    for file_info in benchmarked:
        query = file_info["query"]
        rel_path = file_info["rel_path"]
        raw_tokens = file_info["tokens"]

        depth0 = generate_focus(query, 0)
        if depth0 and has_file_section(depth0, rel_path):
            results["retrieval_hits"] += 1

            pure_summary = extract_interface_section(depth0, rel_path)
            amdb_pure_tokens = count_tokens(pure_summary) if pure_summary else 0
            if raw_tokens > 0 and amdb_pure_tokens > 0:
                compression = (1 - amdb_pure_tokens / raw_tokens) * 100
                results["file_stats"].append(
                    {
                        "name": query,
                        "raw": raw_tokens,
                        "amdb": amdb_pure_tokens,
                        "compression": compression,
                    }
                )

        depth1 = generate_focus(query, 1)
        if depth1:
            if has_graph_edges(depth1):
                results["graph_hits"] += 1
            global_reduction = (1 - count_tokens(depth1) / total_project_tokens) * 100
            results["global_reduction_sum"] += global_reduction

    top_5_heavy = heapq.nlargest(5, results["file_stats"], key=lambda x: x["raw"])

    print("\n" + "=" * 95)
    print("🥊 HEAVYWEIGHT MATCH: Implementation (Raw) vs Interface (AMDB)")
    print("   Target: Top 5 Largest Files (Evaluating Noise Reduction)")
    print("=" * 95)
    print(
        f"{'File Name':<25} | {'Raw Tokens':<10} | {'AMDB Tokens':<12} | {'Compression':<12}"
    )
    print("-" * 95)

    total_heavy_comp = 0
    for match in top_5_heavy:
        print(
            f"{match['name']:<25} | {match['raw']:<10} | {match['amdb']:<12} | {match['compression']:5.1f}%"
        )
        total_heavy_comp += match["compression"]

    total_files = len(benchmarked)
    if total_files == 0:
        print("\n❌ Error: No indexed source files found to benchmark.")
        return

    success_rate = (results["retrieval_hits"] / total_files) * 100
    graph_rate = (results["graph_hits"] / total_files) * 100
    avg_global_reduction = results["global_reduction_sum"] / total_files
    avg_heavy_comp = total_heavy_comp / len(top_5_heavy) if top_5_heavy else 0

    print("\n" + "=" * 60)
    print("🏆 AMDB BENCHMARK SCORECARD (corrected harness)")
    print("=" * 60)

    print("\n1. 🎯 Precision Targeting (Retrieval Accuracy, depth 0)")
    print(f"   - Score: {success_rate:.1f}%  ({results['retrieval_hits']}/{total_files})")
    print("   - Hit = the requested file's own section heading is present.")

    print("\n2. 📉 Global Efficiency (Token Savings, depth 1)")
    print(f"   - Score: {avg_global_reduction:.1f}% Reduction")
    print("   - Focus output tokens vs. full-repo raw tokens.")

    print("\n3. 🗜️ Noise Reduction (Interface Extraction)")
    print(f"   - Score: {avg_heavy_comp:.1f}% Compression (top-5 largest files)")
    print("   - Interface section tokens vs. that file's raw tokens.")

    print("\n4. 🕸️ Graph Presence (depth 1)")
    print(f"   - Score: {graph_rate:.1f}%  ({results['graph_hits']}/{total_files})")
    print("   - Hit = at least one real '-->' edge inside the mermaid block,")
    print("     not merely the presence of a mermaid fence.")

    print("\n" + "=" * 95)
    print("⚖️ COMPARISON: amdb vs raw full-repo dump vs grep-based agentic search")
    print("   Task per question: locate the named symbol's definition and its callers.")
    print("   grep protocol: one grep -rn call, then read every matched file in full.")
    print("=" * 95)

    comparison = run_comparison(file_list, total_project_tokens)

    print(
        f"{'Question (symbol)':<20} | {'amdb tok':>8} | {'calls':>5} | {'raw dump tok':>12} | {'calls':>5} | {'grep tok':>9} | {'calls':>5}"
    )
    print("-" * 95)
    for row in comparison:
        print(
            f"{row['query']:<20} | {row['amdb_tokens']:>8,} | {row['amdb_calls']:>5} | {row['raw_tokens']:>12,} | {row['raw_calls']:>5} | {row['grep_tokens']:>9,} | {row['grep_calls']:>5}"
        )

    avg_amdb = sum(r["amdb_tokens"] for r in comparison) / len(comparison)
    avg_grep = sum(r["grep_tokens"] for r in comparison) / len(comparison)
    avg_grep_calls = sum(r["grep_calls"] for r in comparison) / len(comparison)
    print("-" * 95)
    print(
        f"{'average':<20} | {avg_amdb:>8,.0f} | {1:>5} | {total_project_tokens:>12,} | {1:>5} | {avg_grep:>9,.0f} | {avg_grep_calls:>5.1f}"
    )

    print("\n✅ Benchmark Complete.")


if __name__ == "__main__":
    run_benchmark()
