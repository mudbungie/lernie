#!/usr/bin/env bash
# Wait until an agent is quiescent — no pending inbox messages and no
# executor holding its lease.
#
#   fleet-await.sh <workspace> <agent-id> <timeout-secs>
#
# Quiescence has exactly two observables, both of them harness facts and
# neither of them a sidecar file (ARCH §2.11):
#
#   * `inbox/<agent-id>/` holds no pending message files — everything
#     deposited has been drained into the transcript;
#   * `flock -n` on `inbox/<agent-id>/` succeeds — the executor lock is
#     free, so no driver is stepping the branch.
#
# Both must hold, and then hold again 2s later: a driver that has just
# delivered its last message releases the lease only at the end of the
# exit protocol, and a child's result deposit can revive a parent that
# looked idle a moment earlier. The confirmation poll is what keeps the
# gap between "drained" and "done" from reading as quiescence.
set -euo pipefail

WS="${1:?usage: fleet-await.sh <workspace> <agent-id> <timeout-secs>}"
AGENT="${2:?usage: fleet-await.sh <workspace> <agent-id> <timeout-secs>}"
TIMEOUT="${3:?usage: fleet-await.sh <workspace> <agent-id> <timeout-secs>}"

INBOX="$WS/inbox/$AGENT"
mkdir -p "$INBOX"

quiet() {
  # Any regular file in the inbox dir is a pending deposit.
  if [ -n "$(find "$INBOX" -maxdepth 1 -type f -print -quit 2>/dev/null)" ]; then
    return 1
  fi
  flock -n "$INBOX" true 2>/dev/null
}

deadline=$(( $(date +%s) + TIMEOUT ))
while [ "$(date +%s)" -lt "$deadline" ]; do
  if quiet; then
    sleep 2
    if quiet; then
      exit 0
    fi
  fi
  sleep 3
done

echo "fleet-await: $AGENT still busy after ${TIMEOUT}s" >&2
exit 1
