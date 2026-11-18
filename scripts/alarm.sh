#!/usr/bin/env bash
# alarm.sh — a red unattended run reaches the operator ONCE per streak (bl-f625).
#
#   scripts/alarm.sh red          the last step of a scheduled job, on failure
#   scripts/alarm.sh green        the last step of a scheduled job, on success
#   scripts/alarm.sh --self-test  both directions under a stand-in `gh`
#
# WHY THIS EXISTS. A scheduled workflow has no author to mail: `advisories.yml`
# went red every day for a week and was found by somebody reading run lists for
# another reason (bl-d8c4). An issue that MENTIONS the owner is the one thing
# GitHub mails without a setting anybody has to remember.
#
# THE OPEN ISSUE IS THE STATE, and nothing else is. `red` opens one titled for
# the workflow when none is open and is silent when one is, so a streak is one
# notification however many runs it spans; `green` closes it, which is what
# arms the next streak. Queried, never stored — there is no counter to reset
# and nothing to drift if a person closes the issue by hand (the next red run
# simply says so again, which is right: it is still red).
#
# THE TITLE MATCH IS EXACT AND MADE HERE, not by a search: a search matches
# words, and two workflows whose names share one would silence each other.
set -euo pipefail

title() { printf 'unattended run red: %s' "$GITHUB_WORKFLOW"; }
run_url() { printf '%s/%s/actions/runs/%s' "$GITHUB_SERVER_URL" "$GITHUB_REPOSITORY" "$GITHUB_RUN_ID"; }

# The number of the open issue bearing this workflow's title, or nothing.
open_issue() {
    local n t
    while IFS=$'\t' read -r n t; do
        if [ "$t" = "$(title)" ]; then printf '%s' "$n"; return; fi
    done < <(gh issue list --state open --limit 200 --json number,title \
        --jq '.[] | "\(.number)\t\(.title)"')
}

alarm() {
    local n
    n=$(open_issue)
    case "$1" in
    red)
        [ -n "$n" ] || gh issue create --title "$(title)" --body \
            "@$GITHUB_REPOSITORY_OWNER $(run_url) failed. This is the first red run of the streak; later ones add nothing here, and the next green run closes this."
        ;;
    green)
        [ -z "$n" ] || gh issue close "$n" --comment "$(run_url) is green."
        ;;
    *)
        echo "usage: alarm.sh red|green|--self-test" >&2
        return 2
        ;;
    esac
}

self_test() {
    export GITHUB_WORKFLOW=Advisories GITHUB_REPOSITORY_OWNER=owner \
        GITHUB_SERVER_URL=https://example.invalid GITHUB_REPOSITORY=owner/repo GITHUB_RUN_ID=7
    local open calls failed=0
    # The stand-in: `issue list` answers the fixture, every other call is recorded.
    gh() {
        if [ "$2" = list ]; then printf '%s' "$open"; else calls+="$1 $2 $3;"; fi
    }
    expect() { # NAME VERB FIXTURE WANT
        open=$3 calls=""
        alarm "$2"
        if [ "$calls" = "$4" ]; then echo "ok    $1"; else echo "FAIL  $1: got '$calls', want '$4'"; failed=1; fi
    }
    local mine=$'4\tunattended run red: Advisories\n'
    # A title that CONTAINS this one, and one for another workflow: neither is ours.
    local others=$'5\tunattended run red: Advisories nightly\n6\tunattended run red: store-scan\n'
    expect "red, none open: one issue is opened" red "" "issue create --title;"
    expect "red, only other workflows' open: one issue is opened" red "$others" "issue create --title;"
    expect "red, ours open: silent" red "$others$mine" ""
    expect "green, ours open: it is closed" green "$others$mine" "issue close 4;"
    expect "green, none open: silent" green "" ""
    expect "green, only other workflows' open: theirs are left alone" green "$others" ""
    if alarm sideways 2>/dev/null; then echo "FAIL  an unknown verb was accepted"; failed=1; else echo "ok    an unknown verb is refused"; fi
    return "$failed"
}

if [ "${1:-}" = --self-test ]; then self_test; else alarm "${1:-}"; fi
