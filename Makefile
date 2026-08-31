.PHONY: all build release test coverage lint fmt fmt-check check ci \
        line-cap leak-scan rules-audit deny corpus install-hooks install \
        uninstall icon icon-seats clean

# The build authority. Every gate step has ONE home here, and the pre-commit
# hook calls the same targets — so the hook, a hand-run `make check` and any
# future CI cannot drift into three different definitions of "green".

# Install location for `make install`. Defaults to the XDG-ish user-local
# convention; override for system-wide installs or packaging:
#   make install INSTALL_PREFIX=/usr/local
INSTALL_PREFIX ?= $(HOME)/.local
INSTALL_BIN    := $(INSTALL_PREFIX)/bin
# The freedesktop seats the mark is actually resolved through. `Icon=lernie` in
# the entry resolves by NAME through the hicolor theme, so the SVG's basename
# and the entry's `Icon=` must agree — as must `StartupWMClass` and the app id
# the window hands the toolkit (src/mark.rs). `mark::tests` pins all three.
INSTALL_APPS   := $(INSTALL_PREFIX)/share/applications
INSTALL_THEME  := $(INSTALL_PREFIX)/share/icons/hicolor
INSTALL_ICONS  := $(INSTALL_THEME)/scalable/apps

# Build output root. Exported so every cargo invocation below honours an
# override.
CARGO_TARGET_DIR ?= target
export CARGO_TARGET_DIR

all: check

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

TARPAULIN_PIN := 0.35.2

# The 100% coverage floor. The pin is checked before the run rather than
# after: a 0.35.4+ tarpaulin silently drops inline `#[cfg(test)] mod tests`
# files from the coverable denominator, so an unpinned run reports a weaker
# floor as a pass. See tarpaulin.toml.
coverage:
	@have=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" != "$(TARPAULIN_PIN)" ]; then \
	  echo "tarpaulin $(TARPAULIN_PIN) required (have: $${have:-none}); see tarpaulin.toml" >&2; \
	  echo "  cargo install cargo-tarpaulin --version $(TARPAULIN_PIN) --locked" >&2; \
	  exit 1; \
	fi
	cargo tarpaulin --fail-under 100 --skip-clean --engine llvm --out Stdout

# The complete static gate: the line cap + clippy (which reads Cargo.toml
# [lints]) + the ast-grep rules audit + the supply-chain audit. Every tool is
# pinned so the gate is reproducible — ast-grep 0.44.1 (sgconfig.yml),
# cargo-deny 0.20.2 (deny.toml), toolchain 1.95.0 (rust-toolchain.toml).
#
# `line-cap` goes first because it is milliseconds, and `leak-scan` second
# because it is seconds: a structural violation and a disclosure should both
# fail before the minute-scale tools start.
lint:
	$(MAKE) line-cap
	$(MAKE) leak-scan
	cargo clippy --all-targets -- -D warnings
	$(MAKE) rules-audit
	$(MAKE) deny

# The 300-line hard cap on every tracked source file, inline tests included.
# Docs and config are exempt. THIS TARGET IS THE ONE DEFINITION of the cap and
# of what counts as a source file; the pre-commit hook calls it and restates
# nothing.
#
# It scans the WHOLE TREE, not the staged diff. A hook that walked only the
# staged files would make the cap a sampling rather than an invariant: a file
# that crossed the cap and was never touched again would never be looked at
# again. `git ls-files` reads the INDEX, so a staged addition is covered before
# it is ever committed and a staged deletion is already gone.
#
# The cap is a variable, so the same target answers the design-time question:
# `make line-cap LINE_CAP=199` lists the >=200 pre-split band. That stays a
# hand-run view and never a gate — 300 is a WALL, not a target. A file resting
# ON the wall inverts the rule, firing on whoever touches it next, at the
# moment they are finishing something else, when the cheapest way out is
# exactly the line-shaving the rule forbids. Over the cap? Split along a real
# seam and record it in docs/DESIGN.md; never shave lines to duck the limit.
#
# The empty-set guard is this target's own negative check, the same
# two-direction discipline `rules-audit` holds: a broken pattern or a wrong
# working directory would otherwise enumerate nothing and pass silently, which
# is the exact failure this target exists to end.
LINE_CAP := 300
LINE_CAP_EXEMPT := \.(md|txt|toml|yaml|yml|json|lock)$$|(^|/)(Makefile|LICENSE|\.gitignore|\.githooks/)

line-cap:
	@files=$$(git ls-files | grep -Ev '$(LINE_CAP_EXEMPT)' || true); \
	n=$$(printf '%s\n' "$$files" | grep -c . || true); \
	over=$$(printf '%s\n' "$$files" | while IFS= read -r f; do \
	    { [ -n "$$f" ] && [ -f "$$f" ]; } || continue; \
	    c=$$(wc -l < "$$f"); \
	    [ "$$c" -gt $(LINE_CAP) ] && printf '  %s: %s lines\n' "$$f" "$$c"; \
	    true; \
	  done); \
	if [ "$$n" -eq 0 ]; then \
	  echo "line-cap: enumerated 0 source files — the scan is broken, not the tree" >&2; \
	  exit 1; \
	fi; \
	if [ -n "$$over" ]; then \
	  echo "error: source files over the $(LINE_CAP)-line cap:" >&2; \
	  printf '%s\n' "$$over" >&2; \
	  echo "       split along a real seam (docs/DESIGN.md) — do not shave lines." >&2; \
	  exit 1; \
	fi; \
	echo "line-cap: $$n source files, all within $(LINE_CAP) lines"

# The disclosure gate (bl-e878, ported from yog): no credential, routable
# address, MAC, home path, email, pasted dialogue, agent-session artifact,
# credential-shaped path or unreadable blob in the tree.
# `scripts/leak-rules.sh` is the ONE definition of what counts,
# `scripts/leak-scan.sh` runs it, and this target is the door — neither
# restates the other.
#
# BOTH DIRECTIONS in one target, the same discipline `rules-audit` holds: the
# self-test runs FIRST, and it is the stronger of the two checks. Every rule
# owns a fixture in which every non-comment line must be flagged BY THAT RULE
# and must carry the `notreal` marker, plus `clean.txt`/`clean-paths.txt` of
# near-misses that must NOT be flagged. A leak gate does not die by being
# wrong; it dies by silently matching nothing after a pattern is edited, and
# then passing everything forever — and a gate that cries wolf gets bypassed,
# which is the same death by the other road.
#
# It reads INDEX BLOBS, not the worktree: `git checkout-index` materializes the
# index into a scratch tree and the scan reads that, so the bytes scanned are
# the bytes committed. A leak that is `git add`ed and then overwritten with a
# clean copy on disk is still caught.
#
# The COMMIT MESSAGE is not in any tree, so no pre-commit step can see it;
# `.githooks/commit-msg` runs this same scanner over it. `make install-hooks`
# seats both.
leak-scan:
	@scripts/leak-scan.sh --self-test
	@scripts/leak-scan.sh

# Static audit of every ast-grep rule (rules/, pinned ast-grep 0.44.1 — see
# sgconfig.yml). BOTH DIRECTIONS: `src` must be clean, AND every rule must
# still flag its deliberate violation in rules/fixtures, so a rule whose
# pattern silently stopped matching anything cannot pass as green forever.
#
# PER RULE, NOT PER DIRECTORY (bl-1827). This used to ask only whether
# `rules/fixtures` was flagged by SOMETHING, which nine live rules answer for a
# tenth dead one forever. It now runs each rule ALONE — `--filter` on the `id`
# read out of the rule's own file — and fails the rule that flags nothing. Two
# things follow, and the second is why the change was worth making:
#
#   - a rule that stops matching is named, individually, on the run it breaks;
#   - a rule with NOTHING TO MATCH IN `src` is measurable at all. The four
#     confinement rules are exactly that: lernie has no `unsafe` and no lock at
#     all, and its one child process sits in the file its own rule names, so
#     `ast-grep scan src` is silent about all four whether they work or not,
#     and this loop is the only thing that says they do.
#
# The id is read from the file rather than kept in a list here, so a new rule
# cannot be added to a stale list — and a new rule with no fixture fails on the
# run that adds it. The empty-set guard is the same discipline `line-cap`
# holds: enumerating no rules at all is a broken audit, not a clean tree.
rules-audit:
	ast-grep scan src
	@n=0; for r in rules/*.yml; do \
	  id=$$(sed -n 's/^id:[[:space:]]*//p' "$$r" | head -1); \
	  if [ -z "$$id" ]; then \
	    echo "rules-audit: $$r declares no id" >&2; exit 1; \
	  fi; \
	  n=$$((n + 1)); \
	  if ast-grep scan --filter "^$$id$$" rules/fixtures >/dev/null 2>&1; then \
	    echo "rules-audit: [$$id] flagged NOTHING in rules/fixtures — the rule has" >&2; \
	    echo "             regressed, or it was added without a fixture. Fix the rule" >&2; \
	    echo "             or write the violation; never delete the check." >&2; \
	    exit 1; \
	  fi; \
	done; \
	if [ "$$n" -eq 0 ]; then \
	  echo "rules-audit: enumerated 0 rules — the audit is broken, not the tree" >&2; \
	  exit 1; \
	fi; \
	echo "rules-audit: src clean; all $$n rules flagged their fixture"

# Supply-chain audit (cargo-deny 0.20.2 — see deny.toml): licenses, advisories,
# the TLS-stack bans, and registry-only sources.
deny:
	cargo deny check

# Re-vendor the wire conformance corpus from a yog checkout. There is no
# published artifact and no endpoint (yog's corpus/README.md), so the corpus
# arrives by copy; the script never classifies a reply shape, because the
# directory a frame sits in is THIS repository's assertion about it. A shape
# yog has grown lands in corpus/unreadable/ and shows up in the diff.
#
#   make corpus YOG=../yog
YOG ?= ../yog

corpus:
	@scripts/refresh-corpus.sh "$(YOG)"

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

# The complete gate, and the exact target any CI runs. Cheap steps first.
check: fmt-check lint coverage

ci: check

# Arm this clone's git hooks: one symlink per file in .githooks/, seated in the
# repo's own hooks directory. Symlinks, not copies, so an updated hook is live
# without a re-run.
#
# NOT `core.hooksPath`, which a machine may set globally to a chain hook; a
# per-repo override would silence that machine-wide hook for this repo, while
# seating the links where git already looks keeps both.
#
# Refused from a linked worktree: `bl claim` deletes those, and links pointing
# into one would rot the moment the ball closed.
install-hooks:
	@top=$$(git rev-parse --path-format=absolute --show-toplevel) && \
	common=$$(git rev-parse --path-format=absolute --git-common-dir) && \
	if [ "$$common" != "$$top/.git" ]; then \
	  echo "install-hooks: run this in the main checkout, not a linked worktree" >&2; \
	  exit 1; \
	fi; \
	mkdir -p "$$common/hooks"; \
	for h in .githooks/*; do \
	  ln -sfn "$$top/$$h" "$$common/hooks/$${h#.githooks/}"; \
	done; \
	echo "hooks: seated $$(ls .githooks | tr '\n' ' ')in $$common/hooks"

# Re-emit the checked-in vector source from the generator that defines it
# (src/mark.rs). It is a DERIVATION and never a hand-edit: `mark::tests` asserts
# the tracked file still equals what the generator produces, so an edit made
# here without one made there fails the suite. This is the only sanctioned way
# to move it.
icon:
	@cargo run --quiet --example icon -- assets

# The freedesktop seats, as ONE target both `install` and a hand-run refresh go
# through. `install` runs it because on Wayland the compositor resolves the
# window's mark through the INSTALLED seats (app id -> the desktop entry ->
# Icon=lernie -> the theme), ignoring the icon the binary embeds entirely: a
# rebuilt binary alone can never refresh the mark.
#
# THE INSTALLED ENTRY NAMES AN ABSOLUTE BINARY; THE TRACKED ASSET NEVER DOES.
# `Exec=` is resolved by the desktop environment out of the SESSION's
# environment, not out of a login shell's — so the tracked `Exec=lernie` starts
# nothing at all on a box where the binary's directory is put on `PATH` by a
# shell profile the session never reads, which is the ordinary shape of a
# user-local or cargo bin directory. It fails silently: no window, no message,
# nothing in a log. Only an absolute path is safe, and it cannot live in the
# tracked file — that file is one repository's source for every box,
# `mark::tests` reads it for the three-way spelling agreement, and a real
# absolute path in it would be a disclosure besides. So the substitution
# happens HERE, at seat time, into the INSTALLED COPY ONLY, recomputed on every
# run rather than patched: a box already fixed by hand converges on this same
# answer instead of being edited a second time, and re-running is idempotent.
#
# The ladder is two rungs and a refusal:
#
#   1. `$(INSTALL_BIN)/lernie` — this Makefile's own installation, and it goes
#      FIRST precisely because the defect being fixed is that directory not
#      being on a PATH, INCLUDING the PATH `make` itself was handed.
#   2. `command -v lernie` — a binary some other hand installed elsewhere,
#      which is what the operator actually runs.
#   3. Neither, or a resolution that is not absolute: REFUSE, naming what was
#      looked for. An entry whose `Exec` resolves nowhere is the whole defect,
#      so writing one is not a fallback — it is the bug with a success message.
#
# rename(2) atomicity, for the same reason the binary below gets it: the temp
# name is in the SAME directory, so a launcher reading the entry sees whole-old
# or whole-new and never the window a partial write opens.
#
# The cache rebuild is not decoration. GTK judges a cache at the theme root
# valid while its mtime is >= the toplevel directory's, and installing into an
# existing SUBdirectory never bumps the toplevel — so a third-party
# gtk-update-icon-cache run serves its old index forever. Rebuilding makes the
# fresh file authoritative; the `touch` is the fallback where the tool is
# absent, and it also bumps the mtime a live shell rescans on.
icon-seats:
	@bin="$(INSTALL_BIN)/lernie"; \
	[ -x "$$bin" ] || bin=$$(command -v lernie 2>/dev/null || true); \
	case "$$bin" in /*) ;; *) \
	  echo "icon-seats: no lernie binary to name in the desktop entry." >&2; \
	  echo "  looked at $(INSTALL_BIN)/lernie, then PATH." >&2; \
	  echo "  run 'make install', or point INSTALL_PREFIX at where it lives." >&2; \
	  exit 1;; \
	esac; \
	mkdir -p "$(INSTALL_ICONS)" "$(INSTALL_APPS)"; \
	install -m 0644 assets/lernie.svg "$(INSTALL_ICONS)/lernie.svg"; \
	tmp="$(INSTALL_APPS)/.lernie.desktop.tmp"; \
	sed "s|^Exec=.*|Exec=$$bin|" assets/lernie.desktop > "$$tmp" && \
	  chmod 0644 "$$tmp" && mv -f "$$tmp" "$(INSTALL_APPS)/lernie.desktop"; \
	echo "icon seats: $(INSTALL_ICONS)/lernie.svg, $(INSTALL_APPS)/lernie.desktop (Exec=$$bin)"
	@if command -v gtk-update-icon-cache >/dev/null 2>&1; then \
	  gtk-update-icon-cache -q -f -t "$(INSTALL_THEME)" 2>/dev/null || true; \
	fi
	@touch "$(INSTALL_THEME)"

# rename(2) atomicity on the installed binary: write a temp name in the SAME
# directory, then `mv -f` it into place. Anything holding the old path — a
# desktop launcher, a shell that already resolved it — then sees whole-old or
# whole-new, never the ENOENT window install(1) opens between its unlink and
# its write.
#
# The ORDER is load-bearing and that is why `icon-seats` is a recipe step here
# rather than a prerequisite: the entry it writes names the binary, so the
# binary has to be in place before it looks. As a prerequisite it ran first, and
# on a box installing for the very first time it would find nothing and refuse
# — the fresh install being exactly the case that has to work.
install: release
	@mkdir -p "$(INSTALL_BIN)"
	@install -m 0755 $(CARGO_TARGET_DIR)/release/lernie "$(INSTALL_BIN)/.lernie.tmp" && \
	  mv -f "$(INSTALL_BIN)/.lernie.tmp" "$(INSTALL_BIN)/lernie"
	@echo "installed $(INSTALL_BIN)/lernie"
	@$(MAKE) --no-print-directory icon-seats

uninstall:
	@rm -f "$(INSTALL_BIN)/lernie" \
	       "$(INSTALL_ICONS)/lernie.svg" \
	       "$(INSTALL_APPS)/lernie.desktop"
	@echo "removed the binary and its icon seats"

# There is deliberately NO `publish` target. `Cargo.toml` carries
# `publish = false`: the seat's first release is the coordinated cutover moment
# for the whole four-component split, and it is an operator decision that has
# not been made. `cargo publish` is irreversible — a yanked version stays
# downloadable — and the name carries two eras besides, so a first publish
# under it is the act that fixes the fence in the public record. A convenience
# target for an irreversible act is how the act happens by accident.

clean:
	cargo clean
