#!/usr/bin/env bash
# Live end-to-end for the fleet demo. Real model calls over codex/gpt-5.4
# through the real `bz` — it spends real usage and is deliberately NOT
# wired into `make check`.
#
#   fleet/test.sh
#
# Scenarios A–E each print PASS/FAIL and accumulate the exit code; every
# failure dumps the failing agent's last response tail, its stderr, and its
# branch log, and `lernie scan` runs between scenarios into the log.
set -uo pipefail

FLEET_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FLEET_BIN="$FLEET_DIR/bin"
REPO_ROOT="$(dirname "$FLEET_DIR")"
export FLEET_BIN

# The worktree's own build, never the installed binary: the installed one
# links brazen 0.0.4 and the load-time version guard refuses the bz 0.0.5
# this HEAD pins.
LERNIE_BIN="${LERNIE_BIN:-$REPO_ROOT/target/release/lernie}"
export LERNIE_BIN

SCRATCH="${FLEET_SCRATCH:-/tmp/claude-1000/-home-mark-dev-lernie/d8b3c251-6d27-458f-a55c-166907ca93db/scratchpad/fleet-e2e}"
RUN="$SCRATCH/run-$(date +%Y%m%d-%H%M%S)-$$"
HOME_DIR="$RUN/home"
WS="$RUN/ws"
mkdir -p "$RUN"

# shellcheck source=fleet/bin/fleet-lib.sh
. "$FLEET_BIN/fleet-lib.sh"
# shellcheck source=fleet/bin/fleet-charters.sh
. "$FLEET_BIN/fleet-charters.sh"

say "fleet e2e: run dir $RUN"
say "fleet e2e: building the worktree's own lernie"
(cd "$REPO_ROOT" && cargo build --release) || { say "FAIL build"; exit 1; }

export LERNIE_HOME="$HOME_DIR"

# ---------------------------------------------------------------- A ----
# Bring-up: harness root, workspace, fleet config commit, a coordinator,
# and the two long-lived watchers dispatched off it.
hr "scenario A: bring-up"
if ! "$FLEET_BIN/fleet-up.sh" "$HOME_DIR" "$WS"; then
  fail A "fleet-up.sh failed"
  exit 1
fi

ROOT="$("$LERNIE_BIN" prompt "$WS" "$(bringup_goal)" 2>"$RUN/prompt.stderr")"
if [ -z "$ROOT" ]; then
  fail A "lernie prompt produced no agent id"
  cat "$RUN/prompt.stderr"
  exit 1
fi
say "coordinator agent id: $ROOT"
await_or_dump "$WS" "$ROOT" 300 || fail A "coordinator did not go quiescent after bring-up"

"$LERNIE_BIN" dispatch sensor "$WS" "$ROOT" --goal "$(sensor_charter)" || fail A "dispatch sensor failed"
"$LERNIE_BIN" dispatch shepherd "$WS" "$ROOT" --goal "$(shepherd_charter)" || fail A "dispatch shepherd failed"

SENSOR=""; SHEPHERD=""
for _ in $(seq 1 30); do
  SENSOR="$(child_with_role "$WS" "$ROOT" sensor)" || SENSOR=""
  SHEPHERD="$(child_with_role "$WS" "$ROOT" shepherd)" || SHEPHERD=""
  [ -n "$SENSOR" ] && [ -n "$SHEPHERD" ] && break
  sleep 2
done
say "sensor=$SENSOR shepherd=$SHEPHERD"

if [ -n "$SENSOR" ] && [ -n "$SHEPHERD" ]; then
  pass A "agent refs exist for root + both children, each dispatch commit naming its role"
else
  fail A "missing child ref (sensor='$SENSOR' shepherd='$SHEPHERD')"
  hr "agents/*"; agent_refs "$WS"
  dump_agent "$WS" "$ROOT"
fi

ROOT_TOOLS_BEFORE="$(tools_declared "$WS" "$ROOT" 001 | tr '\n' ' ')"
say "coordinator step 001 declared: $ROOT_TOOLS_BEFORE"

for child in $SENSOR $SHEPHERD; do
  await_or_dump "$WS" "$child" 420 || fail A "$child did not settle"
  if [ -s "$WS/steps/$child/001/response.json" ] &&
    grep -q '"type":"end"' "$WS/steps/$child/001/response.json"; then
    pass A "$child ran step 001 to a terminal end"
  else
    fail A "$child has no terminal step 001 response"
    dump_agent "$WS" "$child"
  fi
done
await_or_dump "$WS" "$ROOT" 420 || say "note: coordinator still busy after child returns"

# The coordinator's grant must survive its children returning. A child's
# dispatch commit prunes `descriptions/**` to its own narrower grant; the
# work-product transfer's `CONTEXT_EXCLUDES` now excludes `descriptions/**`
# from the fork→tip diff it applies (bl-475a), so those deletions no longer
# ride back into the parent. This is the regression guard for that fix.
ROOT_LAST="$(find "$WS/steps/$ROOT" -maxdepth 1 -mindepth 1 -type d | sort | tail -1)"
ROOT_TOOLS_AFTER="$(tools_declared "$WS" "$ROOT" "$(basename "$ROOT_LAST")" | tr '\n' ' ')"
say "coordinator latest step declared: $ROOT_TOOLS_AFTER"
if [ "$ROOT_TOOLS_BEFORE" = "$ROOT_TOOLS_AFTER" ]; then
  pass A "coordinator's declared toolset survived its children's returns"
else
  fail A "coordinator's declared toolset changed across a child return: '$ROOT_TOOLS_BEFORE' -> '$ROOT_TOOLS_AFTER'"
  hr "descriptions/** deletions on the coordinator branch"
  git -C "$WS/repo.git" log --oneline --name-status "agents/$ROOT" -- 'descriptions/*' | head -20
fi
scan_evidence "after A"

# ---------------------------------------------------------------- B ----
# Sensor relay: three channel lines — a peer human, a peer agent, and our
# own coordinator's signed post — one cycle, one EVIDENCE line.
hr "scenario B: sensor relay"
"$FLEET_BIN/fleet-seed-slack.sh" "Matt McCauley" \
  "the hedonic model needs the v2 weights, can your side confirm the pipeline?" >/dev/null
"$FLEET_BIN/fleet-seed-slack.sh" "Matt McCauley" \
  "$(printf 'status: spec section 4 revised\n[Sent using <@Claude>]\n— Likelihood')" >/dev/null
"$FLEET_BIN/fleet-seed-slack.sh" "Mark H" \
  "$(printf 'ack, checking\n[Sent using <@Claude>]\n— Prior')" >/dev/null

if [ -n "$SENSOR" ]; then
  "$FLEET_BIN/fleet-cycle.sh" "$WS" "$SENSOR" "relay-1"
  if await_or_dump "$WS" "$SENSOR" 300; then
    assert_that B "sensor transcript carries a slack_read tool result" \
      transcript_grep "$WS" "$SENSOR" 'latest_ts'
    if inbound_grep "$WS" "$ROOT" "$SENSOR" '^EVIDENCE:'; then
      pass B "coordinator received an EVIDENCE: line from the sensor"
    else
      fail B "no EVIDENCE: line from the sensor reached the coordinator"
      dump_agent "$WS" "$SENSOR"
    fi
    # The structural read-only assertion. The grant is what is structural;
    # a request additionally declares every tool its *inherited* history
    # names (the closure rule, `dispatch/tools.rs`), so the load-bearing
    # half is the absence of the write tool, not the exact set.
    declared="$(tools_declared "$WS" "$SENSOR" 001 | tr '\n' ' ')"
    say "sensor step 001 declared: $declared"
    if printf '%s' "$declared" | grep -q 'slack_read' &&
      printf '%s' "$declared" | grep -q 'message'; then
      pass B "sensor's granted tools (slack_read, message) reached its request"
    else
      fail B "sensor step 001 declares '$declared' — its grant did not reach the wire"
    fi
    if printf '%s' "$declared" | grep -q 'slack_post'; then
      fail B "sensor's request declares slack_post — the one-speaker grant leaked"
    else
      pass B "sensor's request cannot name slack_post — structurally read-only"
    fi
  else
    fail B "sensor did not settle after its cycle"
  fi
else
  fail B "no sensor to cycle"
fi
await_or_dump "$WS" "$ROOT" 420 || say "note: coordinator still busy after the relay"
scan_evidence "after B"

# ---------------------------------------------------------------- C ----
# Builder + work-product transfer: a coordinator charters a builder, the
# builder commits a file, and the harness transfers it into the parent's
# worktree at result delivery.
#
# On a FRESH coordinator, deliberately: one claim per scenario, so a failure
# here can never be scenario A's regression guard tripping instead of this
# scenario's own claim (the transfer).
hr "scenario C: builder + work-product transfer"
ROOT2="$("$LERNIE_BIN" prompt "$WS" "$(charter_builder_goal)" 2>"$RUN/prompt2.stderr")"
say "second coordinator: $ROOT2"
BUILDER=""
if [ -n "$ROOT2" ]; then
  for _ in $(seq 1 30); do
    BUILDER="$(child_with_role "$WS" "$ROOT2" builder)" || BUILDER=""
    [ -n "$BUILDER" ] && break
    sleep 2
  done
fi
if [ -n "$BUILDER" ]; then
  pass C "builder child ref exists ($BUILDER)"
  await_or_dump "$WS" "$BUILDER" 600 || fail C "builder did not settle"
  # A child of the in-model `dispatch` tool used to fork between its
  # parent's `tool_use` commit and the `tool_result` commit that answers it,
  # inheriting a dangling tool_use that got its first model call refused by
  # the provider. Fixed (bl-4231): this is the regression guard.
  if grep -q '"type":"error"' "$WS/steps/$BUILDER/001/response.json" 2>/dev/null; then
    fail C "the tool-dispatched builder's first model call was refused (dangling tool_use)"
    tail -c 600 "$WS/steps/$BUILDER/001/response.json"
  else
    pass C "the tool-dispatched builder's first model call was accepted"
  fi
else
  fail C "no builder child ref"
  hr "agents/*"; agent_refs "$WS"
  [ -n "$ROOT2" ] && dump_agent "$WS" "$ROOT2"
  cat "$RUN/prompt2.stderr"
fi

# The transfer itself, measured on the CLI dispatch path — which forks
# after the parent's step has settled and so carries no dangling tool_use.
# This is the claim `fleet/README.md` maps close/merge onto.
if [ -n "$ROOT2" ]; then
  await_or_dump "$WS" "$ROOT2" 420 || say "note: coordinator busy before the CLI dispatch"
  "$LERNIE_BIN" dispatch builder "$WS" "$ROOT2" --goal "$(builder_goal)" ||
    fail C "CLI dispatch builder failed"
  BUILDER2=""
  for _ in $(seq 1 30); do
    for id in $(agent_refs "$WS"); do
      case "$id" in "$ROOT2"-*) ;; *) continue ;; esac
      [ "$id" = "$BUILDER" ] && continue
      [ "$(agent_role "$WS" "$id")" = builder ] && BUILDER2="$id"
    done
    [ -n "$BUILDER2" ] && break
    sleep 2
  done
  say "cli-dispatched builder: $BUILDER2"
  if [ -n "$BUILDER2" ]; then
    await_or_dump "$WS" "$BUILDER2" 600 || fail C "cli-dispatched builder did not settle"
    await_or_dump "$WS" "$ROOT2" 420 || fail C "coordinator did not settle after the return"
  fi
  if grep -q 'hello from builder' "$WS/agents/$ROOT2/fleet-note.md" 2>/dev/null; then
    pass C "work product transferred into the coordinator worktree at result delivery"
  else
    fail C "no fleet-note.md with the expected content in $WS/agents/$ROOT2/"
    ls -la "$WS/agents/$ROOT2" 2>&1 | head -30
    [ -n "$BUILDER2" ] && dump_agent "$WS" "$BUILDER2"
  fi
fi
conflicted="$(git -C "$WS/repo.git" for-each-ref --format='%(refname)' 'refs/lernie/conflicted/*')"
if [ -z "$conflicted" ]; then
  pass C "no refs/lernie/conflicted/* ref appeared"
else
  fail C "conflicted transfer declined: $conflicted"
fi
scan_evidence "after C"

# ---------------------------------------------------------------- D ----
# Shepherd cycle: the fleet reading, and the report that reaches the
# coordinator (the epitaph deposit carries it).
hr "scenario D: shepherd cycle"
if [ -n "$SHEPHERD" ]; then
  "$FLEET_BIN/fleet-cycle.sh" "$WS" "$SHEPHERD" "sweep-1"
  if await_or_dump "$WS" "$SHEPHERD" 420; then
    if transcript_grep "$WS" "$SHEPHERD" 'silent deaths' ||
      transcript_grep "$WS" "$SHEPHERD" 'drivers launched'; then
      pass D "shepherd transcript carries a bash tool result with scan output"
    else
      fail D "no scan output in the shepherd transcript"
      dump_agent "$WS" "$SHEPHERD"
    fi
    if inbound_grep "$WS" "$ROOT" "$SHEPHERD" '.'; then
      pass D "coordinator received a fleet report from the shepherd"
    else
      fail D "nothing from the shepherd reached the coordinator"
      dump_agent "$WS" "$SHEPHERD"
    fi
  else
    fail D "shepherd did not settle after its cycle"
  fi
else
  fail D "no shepherd to cycle"
fi
await_or_dump "$WS" "$ROOT" 420 || say "note: coordinator still busy after the shepherd report"
scan_evidence "after D"

# ---------------------------------------------------------------- E ----
# One speaker: only the coordinator can reach the channel, and it signs.
hr "scenario E: one speaker"
CHANNEL="$HOME_DIR/slack/channel.ndjson"
# Scenario B seeded a line that already carries the marker and the
# signature, so the verdict has to be the *new* line, never the file's
# contents: an assertion over the whole channel passes on the seed alone.
BEFORE_LINES="$(wc -l <"$CHANNEL")"
# A fresh coordinator again, for the reason scenario C states: the
# bring-up coordinator's `slack_post` descriptor was deleted by its own
# children's returns, so posting from it would re-measure that defect.
ROOT3="$("$LERNIE_BIN" prompt "$WS" "$(post_goal)" 2>"$RUN/prompt3.stderr")"
say "third coordinator: $ROOT3"
if [ -n "$ROOT3" ] && await_or_dump "$WS" "$ROOT3" 600; then
  NEW="$(tail -n +$((BEFORE_LINES + 1)) "$CHANNEL")"
  if printf '%s' "$NEW" | grep -q 'Sent using <@Claude>' &&
    printf '%s' "$NEW" | grep -q 'Prior'; then
    pass E "channel gained a marked, signed coordinator post"
    hr "channel tail"; tail -3 "$CHANNEL"
  else
    fail E "no marked+signed post appended to $CHANNEL"
    hr "channel"; cat "$CHANNEL"
    [ -n "$ROOT3" ] && dump_agent "$WS" "$ROOT3"
  fi
else
  fail E "coordinator did not settle after the post instruction"
  cat "$RUN/prompt3.stderr"
fi
scan_evidence "final"

hr "verdict"
say "run dir: $RUN"
[ "$FLEET_RC" -eq 0 ] && say "fleet e2e: ALL PASS" || say "fleet e2e: FAILURES (see above)"
exit "$FLEET_RC"
