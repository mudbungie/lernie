#!/usr/bin/env bash
# Bring a fleet up: found a harness root, install the mock-Slack tool
# triples into its pools, create a workspace, and author the fleet's
# config commit into it.
#
#   fleet-up.sh <home-dir> <workspace-path>
#
# `$LERNIE_BIN` selects the binary (default: the repo's release build).
# Every step is one of lernie's own front doors — `prime`, `new`,
# `config` — so this script installs no state lernie would not have
# installed itself.
set -euo pipefail

HOME_DIR="${1:?usage: fleet-up.sh <home-dir> <workspace-path>}"
WS_PATH="${2:?usage: fleet-up.sh <home-dir> <workspace-path>}"

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FLEET_SRC="$(dirname "$here")"
REPO_ROOT="$(dirname "$FLEET_SRC")"
LERNIE_BIN="${LERNIE_BIN:-$REPO_ROOT/target/release/lernie}"

[ -x "$LERNIE_BIN" ] || {
  echo "fleet-up: no lernie binary at $LERNIE_BIN (run 'cargo build --release')" >&2
  exit 1
}

mkdir -p "$HOME_DIR"
HOME_DIR="$(cd "$HOME_DIR" && pwd)"
export LERNIE_HOME="$HOME_DIR"

# 1. models.yaml BEFORE prime: `prime` is seed-if-absent, so a file laid
#    down first is the one that survives (ARCH §4.2). The entry shape is
#    copied from the shipped `install/models.yaml`; `provider:` names a
#    brazen provider row, and capabilities/context_window are the facts
#    lernie relies on and brazen does not own.
if [ ! -e "$LERNIE_HOME/models.yaml" ]; then
  cat >"$LERNIE_HOME/models.yaml" <<'YAML'
# Fleet demo models.yaml (ARCH §4.2). One row: every fleet role runs the
# same model, so the demo measures the role separation and not a model mix.
models:
  gpt-5.4:
    provider: codex
    model_id: gpt-5.4
    capabilities: [tool_use_native, streaming]
    context_window: 400000
YAML
fi

# 2. Found the harness root: the `tools/` and `skills/` pools, the
#    `workflows/`/`workspaces/` dirs, and the built-in tool triples.
"$LERNIE_BIN" prime

# 3. Install the mock-Slack triples into the pools, BEFORE `new` — the
#    first config commit snapshots the pools into `descriptions/**`
#    (ARCH §3.3 descriptions-always), so a tool absent from the pool at
#    that moment is grantless for every agent forked off that commit.
install -m 0755 "$FLEET_SRC/tools/lernie-tool-slack_read" "$LERNIE_HOME/tools/lernie-tool-slack_read"
install -m 0755 "$FLEET_SRC/tools/lernie-tool-slack_post" "$LERNIE_HOME/tools/lernie-tool-slack_post"
install -m 0644 "$FLEET_SRC/tools/slack_read.json" "$LERNIE_HOME/tools/slack_read.json"
install -m 0644 "$FLEET_SRC/tools/slack_post.json" "$LERNIE_HOME/tools/slack_post.json"
mkdir -p "$LERNIE_HOME/skills/slack_read" "$LERNIE_HOME/skills/slack_post"
install -m 0644 "$FLEET_SRC/tools/skills/slack_read/SKILL.md" "$LERNIE_HOME/skills/slack_read/SKILL.md"
install -m 0644 "$FLEET_SRC/tools/skills/slack_post/SKILL.md" "$LERNIE_HOME/skills/slack_post/SKILL.md"

# 4. The mock channel. An empty channel file is the ordinary starting
#    state; the tools create it on demand, so this only makes the path
#    obvious to an operator looking for it.
mkdir -p "$LERNIE_HOME/slack"
: >>"$LERNIE_HOME/slack/channel.ndjson"
: >>"$LERNIE_HOME/slack/channel.lock"

# 5. The workspace, with its first config commit off the shipped template.
"$LERNIE_BIN" new "$WS_PATH"

# 6. The fleet's own config commit: roles, budgets, souls. `lernie config`
#    is the only act that advances a config branch (ARCH §2.3), and it
#    refreshes `descriptions/**` from the pools as part of the pass, so the
#    slack triples installed in step 3 land in the snapshot here too.
EDITOR="$here/fleet-config-apply.sh" FLEET_SRC="$FLEET_SRC" \
  "$LERNIE_BIN" config "$WS_PATH"

echo "fleet up: LERNIE_HOME=$LERNIE_HOME workspace=$WS_PATH"
