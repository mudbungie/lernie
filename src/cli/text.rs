//! **What this binary says about itself** — the version line and the usage.
//!
//! Split from [`super`] at the design-time budget, on the seam the file
//! already has: [`super`] is what an invocation *decides*, and this is the
//! prose it says when the decision is to say something about lernie rather than
//! to ask an engine anything. The two change for different reasons — a verb
//! added moves the roster below, a refusal added moves the match above — which
//! is the test that a seam is real.
//!
//! **Both word sections are derived, never restated.** The gestures are
//! [`crate::verbs::help::roster`] and the four structural doors are
//! [`crate::verbs::help::doors`], so a word added to either table is in the
//! usage the moment it is in the table and there is no second list to forget.
//! The doors' paragraphs used to be written out here, beside a `lernie help
//! <verb>` that could reach none of them (bl-6bda) — two homes for one prose,
//! and a page promised for eleven words and kept for seven.

/// The crate's name and version, as the `--version` line.
///
/// **The version is the fence** (yog's `docs/REMOTE.md` §12): `lernie` through
/// 0.0.x was the agent-loop engine, which continues as `litany`; `lernie` at
/// 0.1.0 and above is this seat. So the number printed here is not decoration —
/// it is the one rule that says which program the caller is holding.
pub fn version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

/// The usage text. It states the fence before it states what to type, because
/// the name has two eras in the published record and a caller who typed
/// `lernie` may be holding the other one.
pub fn usage() -> String {
    format!(
        "{}

lernie is the seat: the operator's face on a yog server. It dials in over
mTLS, asks and acts, and paints what comes back. It holds no world, runs no
agent and executes nothing.

THE NAME HAS TWO ERAS. lernie through 0.0.x was the agent-loop engine, which
continues under the name litany. lernie 0.1.0 and above is this seat. The
version is the only rule that separates them.

usage: lernie <verb> [argument…]
       lernie start <workspace> <goal>
       lernie ask <envelope>
       lernie entries
       lernie help [<verb>]
       lernie [--version | --help]

The gestures, typed. Each becomes the envelope the boundary already carries
and goes down the channel its workspace names; the reply stream prints one
envelope per line, and the exit code is 0 when the last reply says ok.

{}

And the four words this binary answers itself, with no channel and no engine
up. Each has its own page too: `lernie help <word>` reaches every entry in
both lists.

{}

  -V, --version   print the name and version
  -h, --help      print this

What it reads, all of it provisioned by hand and none of it ever written by
lernie: this box's own channel at <data root>/wire/, and one channel per
directory under <data root>/wire/workspaces/. The data root is
$XDG_DATA_HOME/lernie, or $HOME/.local/share/lernie.

See docs/DESIGN.md for the role and the module map, and yog's docs/REMOTE.md
for the protocol lernie implements against.",
        version(),
        crate::verbs::help::roster(),
        crate::verbs::help::doors()
    )
}
