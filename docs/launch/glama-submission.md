# Glama submission checklist

Manual steps for listing amdb on Glama so the awesome-mcp-servers PR can carry a real
score badge. Everything below is prepared; nothing has been submitted.

## Pre-flight (already done in this repo)

- `Dockerfile` builds a slim runtime image whose `ENTRYPOINT` is `amdb serve` — the
  container speaks MCP over stdin/stdout the moment it starts, which is exactly what
  Glama's automated check does.
- `serve` starts and answers `initialize` and `tools/list` with **no index present**;
  per-tool calls return a "Run `amdb init` first" tool error, which is expected.
- `.dockerignore` keeps the build context free of `target/`, `.git/`, `.database/`,
  `.amdb/`, and the 128 MB `.fastembed_cache/`.
- `scripts/glama-check.sh` reproduces Glama's check locally with docker. It exits
  non-zero on any failure. (This machine has podman, not docker; the same checks were
  run here with podman — see the task summary for the captured output.)

## Submission steps

1. Go to https://glama.ai/mcp/servers and sign in with the GitHub account that owns
   `BETAER-08/amdb`.
2. Choose to add/claim a server and enter the repository URL:
   `https://github.com/BETAER-08/amdb`.
3. When asked for the Dockerfile path, use the repo-root `Dockerfile` (the default).
   No build args, no custom command — the entrypoint already serves stdio.
4. Wait for Glama's automated inspection to finish and note the server page URL it
   assigns.

Note: https://glama.ai/mcp/connectors is for hosted remote endpoints and does **not**
apply to amdb — amdb is stdio-only.

## After passing: badge for the awesome-mcp-servers PR

Paste this into the amdb entry line in the PR (same placement as neighboring entries,
between the link and the emoji):

```markdown
[![BETAER-08/amdb MCP server](https://glama.ai/mcp/servers/BETAER-08/amdb/badges/score.svg)](https://glama.ai/mcp/servers/BETAER-08/amdb)
```

**Important**: the `BETAER-08/amdb` path segment above is an assumption. Glama assigns
its own server path, which may differ from the GitHub path. Copy the badge markdown (or
at least the path) from the actual Glama server page after listing, and use that.

## Updating the open PR (punkpeye/awesome-mcp-servers#10822)

The PR branch lives on the fork `BETAER-08/awesome-mcp-servers`, branch `add-amdb`.
Edit the amdb line in `README.md` to insert the badge, then:

```bash
git commit -am "Add Glama badge to amdb entry"
git push origin add-amdb
```

The open PR updates automatically; no new PR is needed.

## Owner attention items

- The published image `ghcr.io/betaer-08/amdb:1.0.0` (and `:latest`) was built before
  the entrypoint change: it runs bare `amdb`, so `serve` must be passed as an argument
  there. Images published from the next tag will serve by default. Glama builds from
  the repo Dockerfile, so this does not affect the Glama check — it only means the
  README documents the two invocations differently until the next release.
- Glama listing requires no further repo changes; if its checker flags anything,
  re-run `scripts/glama-check.sh` (or the podman equivalent) to reproduce locally.
