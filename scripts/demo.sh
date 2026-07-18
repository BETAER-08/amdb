#!/usr/bin/env bash
set -euo pipefail

AMDB="${AMDB_BIN:-amdb}"

echo '$ amdb init .'
"$AMDB" init .

echo
echo '$ amdb serve'
echo '  MCP client calls amdb_get_symbol with {"name": "cosine_similarity"}'
echo

{
  printf '%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"demo-client","version":"0.0.0"}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"amdb_get_symbol","arguments":{"name":"cosine_similarity"}}}'
  sleep 1
} | "$AMDB" serve 2>/dev/null | grep '"id":2' | python3 -c '
import json
import sys

msg = json.loads(sys.stdin.readline())
matches = json.loads(msg["result"]["content"][0]["text"])
for m in matches:
    name, file, line = m["name"], m["file"], m["line"]
    print(f"{name} — {file}:{line}")
    sig = m.get("signature")
    if sig:
        print(f"  signature:  {sig}")
    vis = "public" if m["is_public"] else "private"
    print(f"  visibility: {vis}")
    for c in m.get("callers", []):
        cn, cf = c["name"], c["file"]
        print(f"  called by:  {cn} ({cf})")
    callees = ", ".join(c["name"] for c in m.get("callees", []))
    if callees:
        print(f"  calls:      {callees}")
'

echo
echo 'Answer came from the local index. No network. No code left the machine.'
