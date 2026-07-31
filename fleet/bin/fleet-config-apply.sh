#!/usr/bin/env bash
# The `$EDITOR` shim for `lernie config`.
#
# `lernie config` materializes a transient checkout of the config lineage,
# refreshes `descriptions/**` from the data-root pools, and then runs
# `sh -c 'exec $EDITOR "$1"' sh <checkout-dir>` — so `$EDITOR` receives the
# **checkout directory**, not a file path (`src/bin/lernie/cli.rs`,
# `edit_in_editor`). This script is that editor: it drops the fleet's
# control files into the checkout and exits 0, and `lernie config` commits
# whatever changed.
#
#   EDITOR=<this script> FLEET_SRC=<repo>/fleet lernie config <workspace>
#
# `$FLEET_SRC` defaults to the `fleet/` directory this script lives in.
set -euo pipefail

checkout="${1:?usage: fleet-config-apply.sh <config-checkout-dir>}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
src="${FLEET_SRC:-$(dirname "$here")}"

[ -d "$checkout" ] || { echo "fleet-config-apply: no checkout at $checkout" >&2; exit 1; }
[ -d "$src" ] || { echo "fleet-config-apply: no fleet source at $src" >&2; exit 1; }

cp "$src/providers.yaml" "$checkout/providers.yaml"
cp "$src/workflow.yaml" "$checkout/workflow.yaml"
cp "$src/manifest.yaml" "$checkout/manifest.yaml"

mkdir -p "$checkout/souls"
cp "$src"/souls/*.md "$checkout/souls/"

exit 0
