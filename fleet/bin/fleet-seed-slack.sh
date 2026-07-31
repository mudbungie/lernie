#!/usr/bin/env bash
# Append one raw line to the mock Slack channel, as some other speaker.
#
#   fleet-seed-slack.sh <author> <text>
#
# This is the peer fleet and the human peers: traffic that arrives on the
# channel from outside this fleet. Unlike `slack_post` (the coordinator's
# tool) it appends the text **verbatim** — no agent marker is added — so a
# caller seeding an agent post writes the `[Sent using <@Claude>]` marker
# and the `— Persona` signature into the text itself. That is what makes
# SPEC §6.4's classification testable: the marker is content, exactly as
# it is on the real surface.
#
# Same lock and the same strictly-increasing integer timestamps as the
# tools, so a seed racing a post cannot collide with it.
set -euo pipefail

AUTHOR="${1:?usage: fleet-seed-slack.sh <author> <text>}"
TEXT="${2?usage: fleet-seed-slack.sh <author> <text>}"

data_root="${LERNIE_HOME:-${XDG_DATA_HOME:-$HOME/.local/share}/lernie}"
slack_dir="$data_root/slack"
CHANNEL="$slack_dir/channel.ndjson"
lock="$slack_dir/channel.lock"
export CHANNEL AUTHOR TEXT

mkdir -p "$slack_dir"
[ -e "$lock" ] || : >>"$lock"

exec 9>>"$lock"
flock 9

python3 -c '
import json, os, time


def num(value):
    try:
        return int(str(value))
    except (TypeError, ValueError):
        return 0


path = os.environ["CHANNEL"]
highest = 0
try:
    with open(path, "r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except ValueError:
                continue
            highest = max(highest, num(row.get("ts")))
except FileNotFoundError:
    pass

ts = max(highest + 1, int(time.time()))
row = {"ts": str(ts), "author": os.environ["AUTHOR"], "text": os.environ["TEXT"]}
with open(path, "a", encoding="utf-8") as handle:
    handle.write(json.dumps(row) + "\n")
print(ts)
'
