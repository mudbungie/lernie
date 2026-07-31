#!/usr/bin/env bash
# The watcher charters, in one place.
#
# A charter is the whole interface between a dispatcher and a child (ARCH
# §2.8: `goal.md` is pinned at the head of every model call and frozen
# thereafter), so it carries the three things a watcher cannot derive:
# the coordinator's agent id to report to, the workspace path, and the
# role's cycle protocol. `fleet/test.sh` sources this; a real deployment
# can source it too and dispatch by hand.
#
# Sourced, never executed. Each function reads $ROOT, $WS, $LERNIE_BIN.

bringup_goal() {
  cat <<EOF
You are the fleet coordinator for the workspace at $WS. This exchange is bring-up only: do not
dispatch anything and do not post anywhere. Run one bash command, 'echo \$LERNIE_CONV_BRANCH', and
then answer with exactly two lines: your own agent id, and the workspace path.
EOF
}

charter_builder_goal() {
  cat <<EOF
You are the fleet coordinator for the workspace at $WS. Charter a builder now, with the dispatch
tool, role 'builder'. Its goal must tell it: create the file fleet-note.md in your own worktree
containing exactly the line 'hello from builder', commit it, and bark deliverable -> path; the
workspace is $WS. Then answer with the handle the dispatch tool returned. Do not do the builder's
work yourself.
EOF
}

builder_goal() {
  cat <<EOF
Create the file fleet-note.md in your own worktree containing exactly the line 'hello from builder',
commit it with bash, then bark deliverable -> path as your final response. Nothing else.
EOF
}

post_goal() {
  cat <<EOF
You are the fleet coordinator for the workspace at $WS. Post a one-line fleet status to the slack
channel with your slack_post tool, signed '— Prior' on its own final line. Then answer with the
timestamp the tool returned.
EOF
}

sensor_charter() {
  cat <<EOF
You are the sensor for this fleet. Your coordinator is agent $ROOT — address every report to that
exact agent id with the message tool. The workspace is $WS. Your outward surface is the shared
Slack channel your slack_read tool reads; you hold no way to write to it.
Cycle protocol: every message you receive whose text begins 'cycle:' is one cycle. On a cycle:
(1) verify your instrument with one slack_read; (2) pass oldest = the exact last_seen_ts string you
recorded in your previous cycle's final response, omitting oldest on your first cycle;
(3) classify every message returned, by content, per your soul — your own coordinator signs
'— Prior'; (4) send exactly one line to agent $ROOT with the message tool, prefixed 'EVIDENCE:';
(5) end with a short final response recording the counts you saw and the exact last_seen_ts you
carry forward. Handle the one cycle you are given and then end. Do not loop and do not wait for the
next cycle — you will be woken again.
EOF
}

shepherd_charter() {
  cat <<EOF
You are the shepherd for this fleet. Your coordinator is agent $ROOT — address escalations to that
exact agent id with the message tool. The workspace is $WS and the lernie binary is $LERNIE_BIN.
Cycle protocol: every message you receive whose text begins 'cycle:' is one cycle. On a cycle, via
bash: run '$LERNIE_BIN scan $WS', then "git -C $WS/repo.git branch --list 'agents/*'", then
'ls $WS/inbox'. Classify each agent. Nudge with the message tool only where your soul says to, and
escalate to agent $ROOT only for the four triggers in your soul. End the cycle with a short
final-response fleet report that quotes the scan's silent-deaths line verbatim and names each
agent's state. Handle the one cycle you are given and then end. Do not loop — you will be woken
again.
EOF
}
