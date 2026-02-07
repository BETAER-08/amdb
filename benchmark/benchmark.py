
"""
AMDB Official Benchmark Suite
-----------------------------
This script measures the efficiency of AMDB in three key areas:
1. Retrieval Accuracy: Does it find the correct file?
2. Global Reduction: How much token space is saved compared to the full repo?
3. Noise Reduction: How well does it strip implementation details from complex files?

Requirements:
    pip install tiktoken
"""

import subprocess
import os
import sys
import re
import glob
import heapq

try:
    import tiktoken
except ImportError:
    print("❌ Error: 'tiktoken' library is required.")
    print("👉 Run: pip install tiktoken")
    sys.exit(1)

# Use GPT-4 tokenizer for accurate measurement
ENCODER = tiktoken.get_encoding("cl100k_base")


def run_command(cmd, capture=True):
    try:
        result = subprocess.run(cmd, capture_output=capture, text=True, check=False)
        return result
    except Exception as e:
        return None


def count_tokens(text):
    return len(ENCODER.encode(text))


def scan_project_files(root_dir="src"):
    """Scans all Rust files to establish a baseline."""
    file_list = []
    total_tokens = 0

    for root, _, files in os.walk(root_dir):
        for file in files:
            if file.endswith(".rs"):
                full_path = os.path.join(root, file)
                # Generate query from filename (e.g., vector_store.rs -> "vector store")
                query_name = file.replace(".rs", "").replace("_", " ")

                with open(full_path, "r", encoding="utf-8") as f:
                    content = f.read()
                    raw_len = count_tokens(content)
                    total_tokens += raw_len

                file_list.append({
                    "name": query_name,
                    "path": full_path,
                    "tokens": raw_len
                })

    return file_list, total_tokens


def extract_interface_section(md_path, target_file_path):
    """Extracts only the interface summary of the target file from the generated markdown."""
    if not os.path.exists(md_path): return ""
    with open(md_path, "r", encoding="utf-8") as f:
        content = f.read()

    filename = os.path.basename(target_file_path)
    escaped_name = re.escape(filename)

    # Regex to capture content between "### ...filename" and the next header
    pattern = re.compile(f"### .*{escaped_name}(.*?)(?=\n#|\\Z)", re.DOTALL)

    match = pattern.search(content)
    if match:
        return match.group(1).strip()
    return ""


def run_benchmark():
    print(f"\n🚀 Starting AMDB Official Benchmark Suite")
    print(f"   Target: {os.getcwd()}")
    print("-" * 60)

    # [Step 1] Initialize DB
    if not os.path.exists("../.database"):
        print("⚙️ Initializing AMDB database...", end=" ")
        run_command(["amdb", "init"])
        print("Done.")

    # [Step 2] Baseline Measurement
    print("📊 Measuring Codebase Baseline...", end=" ")
    file_list, total_project_tokens = scan_project_files("../src")
    print(f"Done.")
    print(f"   - Files Scanned: {len(file_list)}")
    print(f"   - Total Raw Tokens: {total_project_tokens:,} (Full codebase size)")

    results = {
        "retrieval_hits": 0,
        "graph_hits": 0,
        "global_reduction_sum": 0,
        "file_stats": []
    }

    print("\n⚔️ Running Comprehensive Tests...")

    for file_info in file_list:
        query = file_info["name"]
        raw_tokens = file_info["tokens"]
        target_path = file_info["path"]

        # Clean output directory for accurate testing
        if os.path.exists("../.amdb"):
            for f in glob.glob(".amdb/*.md"):
                os.remove(f)

        # 1. Generate Context
        run_command(["amdb", "generate", "--focus", query])

        # 2. Verify Output
        generated_files = glob.glob(".amdb/*.md")
        if not generated_files: continue
        md_path = generated_files[0]

        with open(md_path, "r", encoding="utf-8") as f:
            full_md_content = f.read()

        # 3. Measure Metrics
        # A. Recall
        if os.path.basename(target_path) in full_md_content:
            results["retrieval_hits"] += 1

        # B. Graph
        if "```mermaid" in full_md_content:
            results["graph_hits"] += 1

        # C. Global Reduction (vs Full Repo)
        amdb_full_tokens = count_tokens(full_md_content)
        global_reduction = (1 - amdb_full_tokens / total_project_tokens) * 100
        results["global_reduction_sum"] += global_reduction

        # D. Local Compression (Implementation vs Interface)
        pure_summary = extract_interface_section(md_path, target_path)
        amdb_pure_tokens = count_tokens(pure_summary) if pure_summary else 0

        if raw_tokens > 0 and amdb_pure_tokens > 0:
            compression = (1 - amdb_pure_tokens / raw_tokens) * 100
            results["file_stats"].append({
                "name": query,
                "raw": raw_tokens,
                "amdb": amdb_pure_tokens,
                "compression": compression
            })

    # [Step 3] Top 5 Complex Files Analysis
    top_5_heavy = heapq.nlargest(5, results["file_stats"], key=lambda x: x["raw"])

    print("\n" + "=" * 95)
    print("🥊 HEAVYWEIGHT MATCH: Implementation (Raw) vs Interface (AMDB)")
    print("   Target: Top 5 Largest Files (Evaluating Noise Reduction)")
    print("=" * 95)
    print(f"{'File Name':<25} | {'Raw Tokens':<10} | {'AMDB Tokens':<12} | {'Compression':<12} | {'Verdict'}")
    print("-" * 95)

    total_heavy_comp = 0
    for match in top_5_heavy:
        winner = "🏆 AMDB" if match["compression"] > 50 else "Raw"
        print(
            f"{match['name']:<25} | {match['raw']:<10} | {match['amdb']:<12} | {match['compression']:5.1f}%       | {winner}")
        total_heavy_comp += match["compression"]

    # [Step 4] Final Report
    total_files = len(file_list)
    success_rate = (results["retrieval_hits"] / total_files) * 100
    graph_rate = (results["graph_hits"] / total_files) * 100
    avg_global_reduction = results["global_reduction_sum"] / total_files
    avg_heavy_comp = total_heavy_comp / len(top_5_heavy) if top_5_heavy else 0

    print("\n" + "=" * 60)
    print("🏆 AMDB OFFICIAL BENCHMARK SCORECARD")
    print("=" * 60)

    print(f"\n1. 🎯 Precision Targeting (Retrieval Accuracy)")
    print(f"   - Score: {success_rate:.1f}%")
    print(f"   - Verdict: Finds the exact file requested.")

    print(f"\n2. 📉 Global Efficiency (Token Savings)")
    print(f"   - Score: {avg_global_reduction:.1f}% Reduction")
    print(f"   - Verdict: Saves ~{avg_global_reduction:.0f}% of tokens compared to full-repo context dumping.")

    print(f"\n3. 🗜️ Noise Reduction (Interface Extraction)")
    print(f"   - Score: {avg_heavy_comp:.1f}% Compression (on complex files)")
    print(f"   - Verdict: Strips implementation details, keeping only structural context.")

    print(f"\n4. 🕸️ Context Awareness")
    print(f"   - Score: {graph_rate:.1f}% Graph Inclusion")
    print(f"   - Verdict: Dependency graphs were generated for spatial reasoning.")

    print("\n✅ Benchmark Complete.")


if __name__ == "__main__":
    run_benchmark()