#!/usr/bin/env bash
# Shared helpers for `fleet/test.sh`: the verdict accumulator, the
# workspace-state readers, and the evidence dump every failure prints.
#
# Sourced, never executed. `FLEET_RC` accumulates the exit code — a
# scenario that fails does not abort the suite, because the later
# scenarios' evidence is what tells you whether the failure was local.

FLEET_RC=0

say() { printf '%s\n' "$*"; }
hr() { printf -- '---- %s ----\n' "$*"; }

pass() { say "PASS  $1: $2"; }
fail() {
  say "FAIL  $1: $2"
  FLEET_RC=1
}

# assert_that <name> <message> <command...> — run the command, report.
assert_that() {
  local name="$1" msg="$2"
  shift 2
  if "$@"; then pass "$name" "$msg"; else fail "$name" "$msg"; fi
}

# agent_refs <workspace> — every agent id with a branch.
agent_refs() {
  git -C "$1/repo.git" for-each-ref --format='%(refname:short)' 'refs/heads/agents/*' |
    sed 's|^agents/||'
}

# agent_role <workspace> <agent-id> — the role from the dispatch commit
# subject (`dispatch: <role> [<id>]`), the single authoritative home
# (src/prompt/role.rs). Empty for a root, whose subject lacks the prefix.
agent_role() {
  git -C "$1/repo.git" log -n 1 --format='%s' -E \
    --grep="^dispatch: .+ \[$2\]$" "agents/$2" 2>/dev/null |
    sed -n 's/^dispatch: \([^ ]*\) .*/\1/p'
}

# child_with_role <workspace> <parent-id> <role> — the first child of
# <parent> whose dispatch commit names <role>.
child_with_role() {
  local ws="$1" parent="$2" want="$3" id
  for id in $(agent_refs "$ws"); do
    case "$id" in "$parent"-*) ;; *) continue ;; esac
    [ "$(agent_role "$ws" "$id")" = "$want" ] && { printf '%s\n' "$id"; return 0; }
  done
  return 1
}

# transcript_grep <workspace> <agent-id> <pattern> — is <pattern> in any
# committed transcript entry on the agent's branch?
transcript_grep() {
  git -C "$1/repo.git" grep -q -e "$3" "agents/$2" -- 'messages/*' 2>/dev/null
}

# inbound_grep <workspace> <recipient> <sender> <pattern> — did the
# recipient receive, from <sender>, a message matching <pattern>? Reads
# both homes a deposit can be in: still pending in the inbox, or already
# drained into the transcript as `messages/NNN-<sender>.md`.
inbound_grep() {
  local ws="$1" to="$2" from="$3" pat="$4"
  if grep -qE "$pat" "$ws/inbox/$to/$from-"*.md 2>/dev/null; then return 0; fi
  local entry
  for entry in $(git -C "$ws/repo.git" ls-tree --name-only "agents/$to" 'messages/' 2>/dev/null); do
    case "$entry" in *"-$from.md") ;; *) continue ;; esac
    git -C "$ws/repo.git" show "agents/$to:$entry" 2>/dev/null | grep -qE "$pat" && return 0
  done
  return 1
}

# tools_declared <workspace> <agent-id> <step-nnn> — the tool names in a
# step's mirrored canonical request, one per line, sorted.
tools_declared() {
  python3 -c '
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as handle:
    req = json.load(handle)
for name in sorted(t.get("name", "") for t in req.get("tools", []) or []):
    print(name)
' "$1/steps/$2/$3/request.json" 2>/dev/null
}

# dump_agent <workspace> <agent-id> — the evidence every failure prints:
# the last step's response tail, its stderr, and the branch's recent log.
dump_agent() {
  local ws="$1" id="$2" last
  hr "evidence: $id"
  last="$(find "$ws/steps/$id" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | sort | tail -1)"
  if [ -n "$last" ]; then
    hr "$last/response.json (tail)"
    tail -c 4000 "$last/response.json" 2>/dev/null || say "(no response.json)"
    hr "$last/stderr.log"
    cat "$last/stderr.log" 2>/dev/null || say "(no stderr.log)"
  else
    say "(no step records under $ws/steps/$id)"
  fi
  hr "git log agents/$id -5"
  git -C "$ws/repo.git" log --oneline "agents/$id" -5 2>&1 || true
  hr "inbox/$id"
  ls -la "$ws/inbox/$id" 2>&1 || true
}

# await_or_dump <workspace> <agent-id> <timeout> — quiescence, with the
# evidence dumped on timeout.
await_or_dump() {
  if "$FLEET_BIN/fleet-await.sh" "$1" "$2" "$3"; then
    return 0
  fi
  dump_agent "$1" "$2"
  return 1
}

# scan_evidence <label> — the standing workspace-health evidence between
# scenarios. Reads the caller's $WS and $LERNIE_BIN.
scan_evidence() {
  hr "lernie scan ($1)"
  "$LERNIE_BIN" scan "$WS" 2>&1 || true
}
