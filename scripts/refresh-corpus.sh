#!/usr/bin/env bash
# Refresh the vendored wire conformance corpus from a yog checkout.
#
# yog generates the corpus from the boundary that IS the protocol authority
# (yog's `corpus/README.md`, REMOTE §3) and commits it. There is no published
# artifact and no endpoint, so a client either vendors the directory or reads a
# checkout at test time. THIS SEAT VENDORS, and the reason is not taste: the
# crate must build and its suite must pass on a box that has never held a yog
# checkout, and a test that reads a path nobody configured is a test that is
# skipped on every box but one. What vendoring costs is staleness, and this
# script plus the corpus test's protocol stamp is what that costs are paid
# with — the stamp fails loudly when yog's PROTOCOL moves past this build's.
#
# WHAT IT DOES NOT DO IS CLASSIFY. A reply fixture's directory is the seat's
# assertion about it (corpus/README.md), and an assertion is a decision this
# repository makes, never one copied in from upstream. So a shape already
# filed goes back to the directory that holds it, and a shape yog has GROWN
# lands in `unreadable/` — the ledger — where it shows up in the diff as a new
# file that says "this build does not read this yet". A silent pass is the one
# outcome the arrangement excludes.
#
#   scripts/refresh-corpus.sh ../yog
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: scripts/refresh-corpus.sh <yog checkout>" >&2
  exit 2
fi

upstream="$1/corpus"
here="$(cd "$(dirname "$0")/.." && pwd)/corpus"

for required in "$upstream/shapes.json" "$upstream/request" "$upstream/reply"; do
  if [ ! -e "$required" ]; then
    echo "refresh-corpus: $required is missing — is $1 a yog checkout?" >&2
    exit 1
  fi
done

# The three assertion directories, in the order a shape is looked for.
classes=(answers refusals unreadable)

# The shape record and the whole request direction ride across verbatim: a
# request has no Read class, so nothing here is the seat's to decide.
cp "$upstream/shapes.json" "$here/shapes.json"
rm -rf "${here:?}/request"
cp -R "$upstream/request" "$here/request"

# A vendored file is one carrying the upstream envelope's `direction` key.
# That is the provenance test the corpus test uses too, so a file the seat
# wrote by hand is never overwritten and never swept.
vendored() { grep -q '"direction"' "$1"; }

added=0
refreshed=0
for fixture in "$upstream"/reply/*.json; do
  shape="$(basename "$fixture")"
  target=""
  for class in "${classes[@]}"; do
    if [ -e "$here/$class/$shape" ]; then target="$class"; break; fi
  done
  if [ -z "$target" ]; then
    target="unreadable"
    added=$((added + 1))
    echo "  new shape ${shape%.json} -> unreadable/ (the ledger; move it when a pane paints it)"
  else
    refreshed=$((refreshed + 1))
  fi
  cp "$fixture" "$here/$target/$shape"
done

# A shape yog RETIRED leaves a vendored file no upstream fixture claims, which
# the corpus test fails on. Sweep it here rather than leaving the operator to
# work out which of two hundred files is the orphan.
swept=0
for class in "${classes[@]}"; do
  for file in "$here/$class"/*.json; do
    vendored "$file" || continue
    if [ ! -e "$upstream/reply/$(basename "$file")" ]; then
      rm "$file"
      swept=$((swept + 1))
      echo "  retired shape $(basename "${file%.json}") -> removed from $class/"
    fi
  done
done

echo "refresh-corpus: $refreshed refreshed, $added new, $swept retired; request/ and shapes.json copied whole"
