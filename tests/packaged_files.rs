//! **What `cargo publish` would ship, read off the real `cargo package --list`
//! — and every path in it must be a class that was ruled in.**
//!
//! Why a test and not a checklist item: this is the one publication question
//! whose answer cannot be recalled. `cargo publish` is irreversible and a
//! yanked version stays downloadable, so a file that ships once has shipped.
//! The sibling crate published its whole tree — operator home paths across four
//! files and three drive-log transcripts — *because its manifest declared no
//! `include` and so packaged everything git tracked*. With no list, this crate
//! packages 298 files, `scripts/leak-fixtures/` among them.
//!
//! The manifest now declares an allowlist, and **an allowlist without a test is
//! a comment**: nothing else notices when a later edit widens it, and the
//! notice would arrive after the version was public.
//!
//! The classes below are a **second statement** of the manifest's policy, which
//! is deliberate and is the only shape that can work. A check that derived its
//! allowlist from the `include` key would widen with it and stay green through
//! the exact edit it exists to catch.
//!
//! **Both directions, because a shape guard dies by matching nothing.**
//! [`the_list_is_not_vacuous`] fails a spawn that answered with a short list,
//! and [`the_allowlist_sees_its_own_violations`] fails an `is_ruled_in` that
//! has quietly become true of everything — including of
//! `scripts/leak-fixtures/clean.txt`, which is not hypothetical: a bare
//! `README.md` include pattern is gitignore-style and unanchored, and upstream
//! measured it shipping that corpus's index out of a list naming no `scripts`
//! entry.
//!
//! **An allowlist fails closed, and the cost of that is a build input added
//! tomorrow silently not shipping.** [`nothing_in_src_is_a_compile_time_embed`]
//! is the answer: `src` carries no `include_bytes!`/`include_str!` today, and
//! the day one appears, its target is a class `include` must gain.
//!
//! **The one bare `Command` outside the confined file, and it is structural.**
//! `rules/no-bare-command.yml` and `rules/no-bare-fork.yml` confine building
//! and forking a child to `src/test_support/mint.rs`, and `make rules-audit`
//! scans `src` — so this file is outside the audit's reach. That is not a
//! loophole taken; it is the only shape available. An integration test is a
//! separate crate linking `lernie` as a dependency, so it cannot reach a
//! `#[cfg(test)]` door in the library at all, and the library has no
//! production spawn for it to reach instead. A bare `Command::new` added
//! anywhere else under `tests/` is on the author who adds it.

// **A test may relax a lint, and this is the one place that reads oddly.** The
// panic family is denied crate-wide and `clippy.toml` relieves test code — but
// clippy decides what is test code by walking up to a `#[test]` function, so a
// helper called BY one is judged as production. Relaxing it in `Cargo.toml`
// would relax it in production, which is the opposite of the point; the
// narrowest home is this file, where every `expect` is a spawn or a read that
// failing means the harness is broken rather than the tree.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root, which is this test target's own manifest directory.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The real answer to *"what would `cargo publish` upload?"*, one path per
/// line.
///
/// `--offline` keeps the guard hermetic — the lockfile is committed and every
/// dependency is resolved by the time a test binary runs. `--allow-dirty` is
/// required because `cargo package` refuses a worktree with uncommitted changes
/// outright, and a claim worktree mid-edit is the normal case for the author
/// this test is addressed to.
fn packaged() -> Vec<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let out = Command::new(cargo)
        .current_dir(root())
        .args(["package", "--list", "--offline", "--allow-dirty"])
        .output()
        .expect("cargo runs");
    assert!(
        out.status.success(),
        "cargo package --list did not answer: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The classes ruled into the published crate: the crate's own source and the
/// two files crates.io renders. `Cargo.toml.orig` and `.cargo_vcs_info.json`
/// are minted by cargo into the tarball and are not tree files at all.
fn is_ruled_in(path: &str) -> bool {
    let named = matches!(
        path,
        "Cargo.toml"
            | "Cargo.lock"
            | "Cargo.toml.orig"
            | ".cargo_vcs_info.json"
            | "README.md"
            | "LICENSE"
    );
    named
        || path
            .strip_prefix("src/")
            .is_some_and(|p| Path::new(p).extension().is_some_and(|kind| kind == "rs"))
}

/// The defect: design commentary, gate apparatus, a corpus of fabricated
/// secrets and the agent guide shipping to crates.io with the binary. Stated as
/// an allowlist so the NEXT file class added to the tree is red here instead of
/// public there.
#[test]
fn no_commentary_or_apparatus_ships() {
    let strays: Vec<String> = packaged().into_iter().filter(|p| !is_ruled_in(p)).collect();
    assert!(
        strays.is_empty(),
        "paths `cargo publish` would upload that no class rules in. A yanked \
         version stays downloadable — widen `include` in Cargo.toml only with \
         a reason, and add the class here:\n{}",
        strays.join("\n")
    );
}

/// The other side of a fail-closed list: the crate must still be a crate.
#[test]
fn the_files_crates_io_and_the_build_need_ship() {
    let list = packaged();
    for needed in [
        "Cargo.toml",
        "Cargo.lock",
        "README.md",
        "LICENSE",
        "src/lib.rs",
        "src/main.rs",
    ] {
        assert!(
            list.iter().any(|p| p == needed),
            "{needed} is not in the packaged list — `include` dropped a file \
             crates.io or the build needs"
        );
    }
}

/// A guard that measured nothing must not read as a pass: a failed spawn or an
/// empty stdout lands here.
#[test]
fn the_list_is_not_vacuous() {
    let list = packaged();
    let sources = list.iter().filter(|p| p.starts_with("src/")).count();
    assert!(
        sources > 60,
        "the packaged list carries {sources} src paths over {} entries — the \
         spawn is broken, not the tree",
        list.len()
    );
}

/// Whether a source file reads something out of the tree at COMPILE time.
fn embeds(text: &str) -> bool {
    text.contains("include_bytes!") || text.contains("include_str!")
}

/// Every `.rs` file under `src`, however deep.
fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("src is readable").flatten() {
        let at = entry.path();
        if at.is_dir() {
            sources(&at, found);
        } else if at.extension().is_some_and(|e| e == "rs") {
            found.push(at);
        }
    }
}

/// **The cost of a fail-closed list, paid.** An asset embedded from `src` would
/// compile here and fail to compile for anyone who downloaded the crate, since
/// `include` carries `src` and nothing else. The sweep is over the tree rather
/// than over a list, so it covers an embed that does not exist yet.
///
/// The two directories the suite reads from disk — `assets/`, `corpus/` — are
/// read at RUN time by `#[cfg(test)]` code, which a published build never
/// compiles. That is the whole reason this list can be as short as it is.
///
/// Both directions: the predicate is shown to bite, and the walk is shown to
/// have walked.
#[test]
fn nothing_in_src_is_a_compile_time_embed() {
    assert!(embeds("include_str!(\"../assets/lernie.svg\")"));
    assert!(embeds("include_bytes!(\"x\")"));
    assert!(!embeds("std::fs::read_to_string(at)"));

    let mut found = Vec::new();
    sources(&root().join("src"), &mut found);
    assert!(found.len() > 60, "the sweep walked {} files", found.len());
    let embedding: Vec<String> = found
        .into_iter()
        .filter(|at| embeds(&std::fs::read_to_string(at).expect("a source file")))
        .map(|at| at.display().to_string())
        .collect();
    assert!(
        embedding.is_empty(),
        "these read the tree at compile time, and `include` carries only \
         `src` — rule the class into Cargo.toml and into `is_ruled_in` \
         above:\n{}",
        embedding.join("\n")
    );
}

/// The negative direction for the restated policy: each excluded class, and the
/// measured unanchored-pattern trap, must be seen as a violation — and the
/// classes that ship must not.
#[test]
fn the_allowlist_sees_its_own_violations() {
    for stray in [
        "docs/DESIGN.md",
        "AGENTS.md",
        "CLAUDE.md",
        "Makefile",
        "deny.toml",
        "clippy.toml",
        "tarpaulin.toml",
        "sgconfig.yml",
        "rust-toolchain.toml",
        "tests/packaged_files.rs",
        "examples/icon.rs",
        "rules/no-bare-command.yml",
        ".github/workflows/ci.yml",
        ".githooks/pre-commit",
        "scripts/leak-scan.sh",
        "scripts/lernie-leak-gate",
        // the unanchored-pattern sighting: a bare `README.md` include pattern
        // ships a file out of the fabricated-secret corpus, and no `scripts`
        // class rules it in
        "scripts/leak-fixtures/clean.txt",
        "corpus/shapes.json",
        "corpus/answers/conversations.json",
        // the mark's seats, which a `cargo install` runs no `make` to lay down
        "assets/lernie.svg",
        "assets/lernie.desktop",
        // a non-Rust file smuggled under src
        "src/notes.txt",
    ] {
        assert!(!is_ruled_in(stray), "{stray} must not be ruled in");
    }
    for shipped in [
        "src/main.rs",
        "src/lib.rs",
        "src/ui/model/claim.rs",
        "LICENSE",
    ] {
        assert!(is_ruled_in(shipped), "{shipped} must be ruled in");
    }
}
