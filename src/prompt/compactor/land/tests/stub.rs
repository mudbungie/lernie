//! The git-op error arms of the compaction landing, routed through a
//! scripted stub keyed on the git subcommand, so every `map_err` in
//! [`span`], [`base`], and [`replay`] is reachable without constructing
//! the corresponding real-git failure. The behavioral arms live in
//! [`super`].

use super::*;
use std::cell::RefCell;
use std::path::PathBuf;

/// Scripted git: captures answer by subcommand; `fail_run` fails the
/// first `run` whose argv (joined) contains the pattern; `rebase_fails`
/// fails that many non-`--abort` rebase invocations (each stop then
/// consults `ls_files`); `fail_capture` fails the first capture whose
/// argv contains the pattern.
#[derive(Default)]
struct Script {
    fail_run: Option<&'static str>,
    fail_capture: Option<&'static str>,
    rebase_fails: RefCell<u32>,
    ls_files: &'static str,
    /// Answers for the two product `diff` captures (D-class, then
    /// summary A-class).
    deletions: &'static str,
    summaries: &'static str,
    /// Answer for the landed-since grep `rev-list` (empty = no landing).
    landed: &'static str,
    /// `log` answer (the compactor dispatch sha / checkpoint origin).
    log: &'static str,
}

impl Script {
    fn ok() -> Self {
        Self {
            ls_files: "",
            deletions: "messages/001.md\n",
            summaries: "summary/001.md\n",
            landed: "",
            log: "dsha",
            ..Self::default()
        }
    }
    fn land(&self) -> Result<LandOutcome, Error> {
        super::super::land(&PathBuf::from("/x"), "p1", "p1-cmp", self)
    }
}

impl GitRunner for Script {
    fn run(&self, _d: &Path, args: &[&str]) -> std::io::Result<()> {
        let joined = args.join(" ");
        if let Some(pat) = self.fail_run
            && joined.contains(pat)
        {
            return Err(std::io::Error::other("stub fail"));
        }
        // Both the initial `rebase --onto` and each `rebase --continue`
        // count as stops; `rebase --abort` never does.
        if joined.contains("rebase") && !joined.contains("--abort") {
            let mut left = self.rebase_fails.borrow_mut();
            if *left > 0 {
                *left -= 1;
                return Err(std::io::Error::other("rebase stop"));
            }
        }
        Ok(())
    }
    fn run_capture(&self, _d: &Path, args: &[&str]) -> std::io::Result<String> {
        let joined = args.join(" ");
        if let Some(pat) = self.fail_capture
            && joined.contains(pat)
        {
            return Err(std::io::Error::other("stub fail"));
        }
        Ok(match args.first().copied() {
            Some("log") => self.log.into(),
            Some("rev-parse") => "psha".into(),
            Some("rev-list") if joined.contains("--count") => "1".into(),
            Some("rev-list") if joined.contains("--grep") => self.landed.into(),
            Some("rev-list") => "".into(),
            Some("diff") if joined.contains("--diff-filter=D") => self.deletions.into(),
            Some("diff") => self.summaries.into(),
            Some("write-tree") => "tsha".into(),
            Some("commit-tree") => "bsha".into(),
            Some("ls-files") => self.ls_files.into(),
            _ => String::new(),
        })
    }
}

fn assert_op(err: Error, want: &str) {
    match err {
        Error::Git { op, .. } => assert_eq!(op, want),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_point_rev_parse_failure_surfaces() {
    let s = Script {
        fail_capture: Some("rev-parse"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land point");
}

#[test]
fn a_span_rev_list_failure_surfaces() {
    let s = Script {
        fail_capture: Some("rev-list"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land span rev-list");
}

#[test]
fn a_merges_rev_list_failure_surfaces() {
    let s = Script {
        fail_capture: Some("--merges"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land span rev-list");
}

#[test]
fn an_unreachable_point_is_superseded() {
    // `merge-base --is-ancestor` answers by exit code; non-zero is "no".
    let s = Script {
        fail_run: Some("merge-base"),
        ..Script::ok()
    };
    assert_eq!(s.land().unwrap(), LandOutcome::Superseded);
}

#[test]
fn a_landing_since_the_point_is_superseded() {
    let s = Script {
        landed: "somesha\n",
        ..Script::ok()
    };
    assert_eq!(s.land().unwrap(), LandOutcome::Superseded);
}

#[test]
fn a_product_diff_failure_surfaces() {
    let s = Script {
        fail_capture: Some("diff"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land product diff");
}

#[test]
fn scratch_worktree_and_mint_failures_surface() {
    for (pat, op) in [
        ("worktree add", "compaction land scratch worktree"),
        ("read-tree", "compaction land read-tree"),
        ("rm --cached", "compaction land apply deletions"),
        ("restore", "compaction land stage summary"),
    ] {
        let s = Script {
            fail_run: Some(pat),
            ..Script::ok()
        };
        assert_op(s.land().unwrap_err(), op);
    }
    for (pat, op) in [
        ("write-tree", "compaction land write-tree"),
        ("commit-tree", "compaction land commit-tree"),
        ("--count", "compaction land replay count"),
    ] {
        let s = Script {
            fail_capture: Some(pat),
            ..Script::ok()
        };
        assert_op(s.land().unwrap_err(), op);
    }
}

#[test]
fn a_clean_scripted_landing_lands() {
    assert_eq!(Script::ok().land().unwrap(), LandOutcome::Landed);
}

#[test]
fn a_rebase_failure_with_no_conflict_aborts_and_surfaces() {
    // A rebase that failed for a non-conflict reason (dirty tree, bad
    // ref): nothing unmerged, so the landing aborts and surfaces the
    // rebase's own failure.
    let s = Script {
        rebase_fails: RefCell::new(1),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land rebase");
}

#[test]
fn an_unmerged_listing_failure_surfaces() {
    let s = Script {
        rebase_fails: RefCell::new(1),
        fail_capture: Some("ls-files"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land unmerged");
}

// Stages 1+3 on `code.txt`, plus two lines the parser must skip over:
// one with no tab separator, one with a stage token outside 1/2/3.
const MODIFY_DELETE: &str =
    "100644 aaa 1\tcode.txt\nnot a stage line\n100644 eee 9\tcode.txt\n100644 ccc 3\tcode.txt\n";
const BOTH_SIDES: &str = "100644 bbb 2\tsummary/001.md\n100644 ccc 3\tsummary/001.md\n";

#[test]
fn a_live_branch_wins_stop_resolves_and_continues() {
    let s = Script {
        rebase_fails: RefCell::new(1),
        ls_files: MODIFY_DELETE,
        ..Script::ok()
    };
    assert_eq!(s.land().unwrap(), LandOutcome::Landed);
}

#[test]
fn a_live_branch_wins_add_failure_surfaces() {
    // `"add -- "` matches the replay's `add -- <path>` and never the
    // base build's `worktree add --no-checkout`.
    let s = Script {
        rebase_fails: RefCell::new(1),
        ls_files: MODIFY_DELETE,
        fail_run: Some("add -- "),
        ..Script::ok()
    };
    assert_op(
        s.land().unwrap_err(),
        "compaction land live-branch-wins add",
    );
}

#[test]
fn more_stops_than_commits_aborts_rather_than_spins() {
    // One commit to replay (`--count` → 1) but endless conflict stops:
    // git is not making progress, so the landing aborts loudly.
    let s = Script {
        rebase_fails: RefCell::new(99),
        ls_files: MODIFY_DELETE,
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land rebase");
}

#[test]
fn a_both_sides_conflict_declines_with_the_paths() {
    let s = Script {
        rebase_fails: RefCell::new(1),
        ls_files: BOTH_SIDES,
        ..Script::ok()
    };
    assert_eq!(
        s.land().unwrap(),
        LandOutcome::Conflicted(vec!["summary/001.md".to_string()])
    );
}

#[test]
fn a_decline_abort_failure_surfaces() {
    let s = Script {
        rebase_fails: RefCell::new(1),
        ls_files: BOTH_SIDES,
        fail_run: Some("--abort"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land abort");
}

#[test]
fn a_decline_mark_failure_surfaces() {
    let s = Script {
        rebase_fails: RefCell::new(1),
        ls_files: BOTH_SIDES,
        fail_run: Some("update-ref"),
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land decline update-ref");
}

#[test]
fn a_missing_dispatch_commit_is_declined() {
    let s = Script {
        log: "",
        ..Script::ok()
    };
    assert_op(s.land().unwrap_err(), "compaction land dispatch commit");
}
