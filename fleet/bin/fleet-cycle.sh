#!/usr/bin/env bash
# Deposit one cycle wake-up into a watcher's inbox.
#
#   fleet-cycle.sh <workspace> <agent-id> [note]
#
# SPEC.md's watchers run a ~60s / ~90s poll loop of their own. lernie has
# no clock and no watcher (ARCH §8: an idle workspace stays unswept until
# the next touch, by design), so the loop is inverted: something outside
# the harness deposits a cycle message, the agent wakes, does exactly one
# cycle, and goes quiescent again. `lernie message` is that deposit — it
# writes the file and, finding the recipient quiescent, launches the
# driver that delivers it.
#
# In a real deployment the caller is cron:
#
#   * * * * * .../fleet-cycle.sh /path/to/ws <sensor-id>  >/dev/null 2>&1
#
# In the test suite the caller is the test.
set -euo pipefail

WS="${1:?usage: fleet-cycle.sh <workspace> <agent-id> [note]}"
AGENT="${2:?usage: fleet-cycle.sh <workspace> <agent-id> [note]}"
NOTE="${3:-}"

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$(dirname "$here")")"
LERNIE_BIN="${LERNIE_BIN:-$REPO_ROOT/target/release/lernie}"

if [ -z "$NOTE" ]; then
  # No note: the sequence number is the note. It is derived from what the
  # agent has already been sent, so nothing has to store a counter.
  NOTE="$(find "$WS/steps/$AGENT" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l)"
fi

exec "$LERNIE_BIN" message "$WS" "$AGENT" "cycle: $NOTE"
