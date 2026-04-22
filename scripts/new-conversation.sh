#!/usr/bin/env bash
# Scaffold a conversation repo by copying the template and initializing git.
#
# Usage: scripts/new-conversation.sh <template-dir> <dest-dir>
#
# Refuses if <dest-dir> already exists so the caller cannot accidentally
# stomp an existing conversation. `lernie new` (bl-2904) will supersede this
# script with an in-process implementation that embeds the template; for
# v0.1 the shell version is the tool of record.

set -euo pipefail

# When this script runs from inside a git hook (e.g. pre-commit tarpaulin),
# git exports GIT_DIR / GIT_INDEX_FILE / GIT_WORK_TREE pointing at the
# outer repo. Child `git` invocations would inherit those and operate on
# the wrong repo — so unset them before we touch the new conversation.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY \
      GIT_PREFIX GIT_COMMON_DIR GIT_ALTERNATE_OBJECT_DIRECTORIES

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <template-dir> <dest-dir>" >&2
  exit 64
fi

src="$1"
dest="$2"

if [ ! -d "$src" ]; then
  echo "error: template dir $src does not exist or is not a directory" >&2
  exit 1
fi
if [ ! -f "$src/.agent/version" ]; then
  echo "error: $src does not look like a conversation template (missing .agent/version)" >&2
  exit 1
fi
if [ -e "$dest" ]; then
  echo "error: destination $dest already exists; refusing to overwrite" >&2
  exit 1
fi

mkdir -p "$dest"
# cp -a preserves the dotfile tree (.agent/, .gitignore, .gitkeep files).
# The trailing /. copies contents rather than nesting the source dir name.
cp -a "$src/." "$dest/"

git -C "$dest" init --quiet -b main
git -C "$dest" add -A
git -C "$dest" commit --quiet -m "init conversation repo"

echo "created $dest"
