#!/bin/sh
# The regression half of the seat reconciler (bl-155a) — `make deploy-selftest`.
#
# **It drives the REAL `lernie-update`**, end to end, under a fake `curl` and a
# fake `cargo` on `PATH` and a scratch `HOME`. Nothing here re-implements the
# decision: a self-test that restated the rule would prove only that the copy
# still agrees with itself, which is the failure the leak gate's own self-test
# is written to avoid. Every assertion below is about what the shipped file did.
#
# It touches no machine and needs no network, no registry, no toolchain and no
# release, which is what lets it run in the gate — and that is the point. A
# reconciler is unattended code on somebody's desktop, so the failures that
# matter are the quiet ones: it stops installing, or it starts installing the
# wrong thing, and nobody finds out for a month.
#
# BOTH DIRECTIONS, and that is the shape of the table rather than a footnote.
# Half the cases assert an install HAPPENED and with exactly which arguments;
# the other half assert `cargo` was never invoked at all. A reconciler that
# installs on every tick and one that has quietly stopped are both broken, and
# only one of them is loud.
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
SCRIPT="$HERE/lernie-update"
[ -x "$SCRIPT" ] || { echo "deploy-selftest: $SCRIPT is not executable" >&2; exit 1; }

fails=0
cases=0

# The synthetic index. Real sparse-index lines carry a full `deps` array and a
# checksum; the reconciler reads `"vers"` and `"yanked"` by parameter expansion
# and nothing else, so the fixture keeps the one field a greedy match could run
# past — a dependency's `"req"` — and omits the rest.
line() { # <version> <yanked:true|false>
    printf '{"name":"lernie","vers":"%s","deps":[{"name":"egui","req":"^0.29"}],' "$1"
    printf '"features":{},"yanked":%s}\n' "$2"
}

current()    { line 0.0.13 false; line 0.1.4 false; line 0.1.5 false; }
tip_yanked() { line 0.0.13 false; line 0.1.4 false; line 0.1.5 true;  }
seat_gone()  { line 0.0.13 false; line 0.1.4 true;  line 0.1.5 true;  }
all_yanked() { line 0.0.13 true;  line 0.1.4 true;  line 0.1.5 true;  }

# Run the reconciler in a built world and hand the result to a checker, which
# reads four globals: `$code` its exit status, `$out` its combined output,
# `$log` the fake cargo's recorded argv (empty when it was never called), and
# `$root` the install root that world's `$HOME` implies.
run_case() { # <label> <index-fixture-fn> <installed-version-or-empty> <checker-fn>
    _label=$1 _fixture=$2 _have=$3 _check=$4
    cases=$((cases + 1))
    _work=$(mktemp -d "${TMPDIR:-/tmp}/lernie-selftest.XXXXXX")
    mkdir -p "$_work/bin" "$_work/home/.local/bin"
    "$_fixture" > "$_work/index.txt"
    root="$_work/home/.local"

    # `curl`: serves the fixture for the sparse-index URL and refuses anything
    # else with curl's own exit 22, so a reconciler that started fetching a
    # second thing fails here rather than passing silently.
    cat > "$_work/bin/curl" <<EOF
#!/bin/sh
for a in "\$@"; do :; done
[ "\$a" = https://index.crates.io/le/rn/lernie ] \
    || { echo "fake curl: unexpected URL \$a" >&2; exit 22; }
exec cat "$_work/index.txt"
EOF
    # `cargo`: records its whole argv, then emulates the install by writing the
    # binary the reconciler re-reads afterwards. Emulating it is what makes the
    # final report line an assertion rather than a hope.
    cat > "$_work/bin/cargo" <<EOF
#!/bin/sh
printf '%s\n' "\$*" >> "$_work/cargo.log"
_root=; _vers=
while [ \$# -gt 0 ]; do
  case \$1 in --root) _root=\$2; shift ;; --version) _vers=\$2; shift ;; esac
  shift
done
mkdir -p "\$_root/bin"
printf '#!/bin/sh\necho "lernie %s"\n' "\$_vers" > "\$_root/bin/lernie"
chmod 0755 "\$_root/bin/lernie"
EOF
    chmod 0755 "$_work/bin/curl" "$_work/bin/cargo"

    if [ -n "$_have" ]; then
        printf '#!/bin/sh\necho "lernie %s"\n' "$_have" > "$root/bin/lernie"
        chmod 0755 "$root/bin/lernie"
    fi

    set +e
    out=$(HOME="$_work/home" PATH="$_work/bin:$PATH" "$SCRIPT" 2>&1)
    code=$?
    set -e
    log=$(cat "$_work/cargo.log" 2>/dev/null || true)

    if "$_check"; then
        printf '  ok    %s\n' "$_label"
    else
        printf '  FAIL  %s (exit %s)\n' "$_label" "$code"
        printf '%s\n' "$out" | sed 's/^/          | /'
        printf '        cargo: %s\n' "${log:-<never invoked>}"
        fails=1
    fi
    rm -rf "$_work"
}

# The exact argument vector, not merely "cargo ran". `--version` with `--force`
# IS the yank lever's mechanism — without both, cargo refuses to move backwards
# and a rollback silently does nothing — and `--root` is what keeps the new
# build on the path the desktop entry names. A reconciler that dropped any of
# the three would still install, and would still be broken.
installed_vector() { # <version>
    [ "$log" = "install lernie --root $root --locked --version $1 --force" ]
}
installed_5() {
    [ "$code" = 0 ] && installed_vector 0.1.5 \
        && printf '%s' "$out" | grep -q 'installed 0\.1\.5; the next window launch runs it'
}
installed_5_from_absent() {
    installed_5 && printf '%s' "$out" | grep -q 'installing 0\.1\.5 (was absent)'
}
rolled_back_to_4() {
    [ "$code" = 0 ] && installed_vector 0.1.4 \
        && printf '%s' "$out" | grep -q 'installing 0\.1\.4 (was 0\.1\.5)'
}
# The negative arm, and it is an assertion about a NON-event: the recorder file
# was never written, so nothing was installed.
no_install() {
    [ "$code" = 0 ] && [ -z "$log" ] \
        && printf '%s' "$out" | grep -q 'installed 0\.1\.5 is current'
}
refused_with() { # <message-pattern>
    [ "$code" != 0 ] && [ -z "$log" ] && printf '%s' "$out" | grep -q "$1"
}
refused_fence()   { refused_with 'below the 0\.1\.0 fence'; }
refused_empty()   { refused_with 'named no live version'; }
refused_offline() { refused_with 'cannot reach the registry index'; }

echo 'deploy-selftest: driving lernie-update under fake curl/cargo'

# It installs when the registry is ahead, and it installs the newest LIVE one.
run_case 'behind the registry -> installs the newest live version' \
    current 0.1.4 installed_5
# A box with no binary at all is that same question with an empty answer rather
# than a special case: `installed` yields the empty string and the compare
# differs, so the bootstrap and the upgrade are one path.
run_case 'no binary at all -> installs, reporting "was absent"' \
    current '' installed_5_from_absent
# The negative arm. Nothing to do must mean nothing done.
run_case 'already newest -> does NOT invoke cargo' \
    current 0.1.5 no_install
# The rollback lever, whole: a yank makes the previous version newest-live, the
# compare sees it differ from what is installed, and the install goes BACKWARDS.
run_case 'newest yanked -> rolls the box back a version' \
    tip_yanked 0.1.5 rolled_back_to_4
# The era fence. With every seat release yanked the newest live version is the
# agent-loop engine's, and installing it would replace the operator's window
# with a different program that happens to share its name.
run_case 'every seat release yanked -> refuses the engine-era version' \
    seat_gone 0.1.5 refused_fence
run_case 'nothing live at all -> refuses, installs nothing' \
    all_yanked 0.1.5 refused_empty

# The registry unreachable. A refusing `curl` and an EMPTY body are different
# failures and must not report as one, so this drives the first: the shim exits
# 22 the way curl does, and the reconciler must name the registry rather than
# the index's contents. The fixture is the shim itself, so it takes no case
# through `run_case`.
cases=$((cases + 1))
work=$(mktemp -d "${TMPDIR:-/tmp}/lernie-selftest.XXXXXX")
mkdir -p "$work/bin" "$work/home"
printf '#!/bin/sh\nexit 22\n' > "$work/bin/curl"
printf '#!/bin/sh\nprintf "%%s\\n" "$*" >> "%s/cargo.log"\n' "$work" > "$work/bin/cargo"
chmod 0755 "$work/bin/curl" "$work/bin/cargo"
set +e
out=$(HOME="$work/home" PATH="$work/bin:$PATH" "$SCRIPT" 2>&1); code=$?
set -e
log=$(cat "$work/cargo.log" 2>/dev/null || true)
if refused_offline; then
    printf '  ok    %s\n' 'registry unreachable -> refuses, installs nothing'
else
    printf '  FAIL  %s (exit %s)\n' 'registry unreachable -> refuses, installs nothing' "$code"
    printf '%s\n' "$out" | sed 's/^/          | /'
    fails=1
fi
rm -rf "$work"

# The empty-set guard, the same two-direction discipline `make line-cap` and
# `make rules-audit` hold: a table that ran no case is a broken harness, not a
# clean reconciler, and it must not pass as green.
[ "$cases" -gt 0 ] \
    || { echo 'deploy-selftest: ran 0 cases — the harness is broken' >&2; exit 1; }
[ "$fails" = 0 ] || { echo 'deploy-selftest: the reconciler is wrong' >&2; exit 1; }
echo "deploy-selftest: $cases cases, all passed"
