#!/usr/bin/env bash
set -euo pipefail

docker build -t amdb-glama-check .

INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"glama-check","version":"1.0"}}}'

INIT_RESPONSE=$(printf '%s\n' "$INIT_REQUEST" | docker run -i --rm amdb-glama-check)
echo "$INIT_RESPONSE"
echo "$INIT_RESPONSE" | grep -q '"protocolVersion"'
echo "$INIT_RESPONSE" | grep -q '"name":"amdb"'

TOOLS_RESPONSE=$({
  printf '%s\n' \
    "$INIT_REQUEST" \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
  sleep 2
} | docker run -i --rm amdb-glama-check)
echo "$TOOLS_RESPONSE"
for tool in amdb_get_context amdb_focus amdb_get_symbol; do
  echo "$TOOLS_RESPONSE" | grep -q "\"$tool\""
done

echo "glama-check: all checks passed"
