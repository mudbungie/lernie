use super::*;
use std::cell::RefCell;
use tempfile::TempDir;

// --- check_dest --------------------------------------------------
#[test]
fn check_dest_allows_missing_path() {
    let holder = TempDir::new().unwrap();
    let missing = holder.path().join("nope");
    check_dest(&missing).unwrap();
}

#[test]
fn check_dest_allows_empty_directory() {
    let holder = TempDir::new().unwrap();
    check_dest(holder.path()).unwrap();
}

#[test]
fn check_dest_rejects_non_empty_directory() {
    let holder = TempDir::new().unwrap();
    fs::write(holder.path().join("occupant"), b"x").unwrap();
    let err = check_dest(holder.path()).unwrap_err();
    assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
}

#[test]
fn check_dest_surfaces_other_io_errors() {
    // read_dir on a regular file fails with kind != NotFound, so the
    // third arm of check_dest fires.
    let holder = TempDir::new().unwrap();
    let file = holder.path().join("actually-a-file");
    fs::write(&file, b"not a dir").unwrap();
    let err = check_dest(&file).unwrap_err();
    assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
}

// --- scaffold orchestration via stub GitRunner -------------------
/// Records every `git` subprocess run (including the `dest` arg so
/// tests can confirm it executed inside `root/`) and can be programmed
/// to fail at a chosen run index.
struct StubGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail_at: Option<usize>,
}

impl StubGit {
    fn ok() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: None,
        }
    }
    fn failing_at(idx: usize) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: Some(idx),
        }
    }
}

impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        let mut runs = self.runs.borrow_mut();
        let idx = runs.len();
        runs.push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if self.fail_at == Some(idx) {
            Err(io::Error::other(format!("stub fail at {idx}")))
        } else {
            Ok(())
        }
    }

    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        // The stub's checkout always reports dirty, so scaffold's commit
        // step runs (the empty-stage decline is `authoring`'s case).
        self.run(dest, args).map(|()| match args.first() {
            Some(&"status") => "A  version".to_string(),
            _ => String::new(),
        })
    }
}

#[test]
fn scaffold_happy_path_authors_the_first_config_commit() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("ws");
    let git = StubGit::ok();
    scaffold(&dest, &crate::test_support::bare_roots(holder.path()), &git).unwrap();

    let repo = dest.join("repo.git");
    let author = dest.join(".config-author");

    // The control files were written into the authoring checkout — the
    // config commit's tree (§2.2). (Stub git does not remove the
    // checkout, so its contents are observable here.)
    for f in [
        "manifest.yaml",
        "workflow.yaml",
        "providers.yaml",
        "version",
        "souls/worker.md",
        "souls/compactor.md",
    ] {
        assert!(author.join(f).is_file(), "missing {f}");
    }

    // Git sequence (§2.2): bare init with config/default as the initial
    // branch (no `main` is ever created), the orphan authoring
    // checkout, the staged-anything question, the config commit, and the
    // checkout teardown.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 6);
    assert_eq!(runs[0].0, repo);
    assert_eq!(runs[0].1, vec!["init", "--bare", "-b", "config/default"]);
    assert_eq!(runs[1].0, repo);
    assert_eq!(runs[1].1[..4], ["worktree", "add", "--orphan", "-b"],);
    assert_eq!(runs[1].1[4], "config/default");
    assert_eq!(runs[1].1[5], author.to_string_lossy().to_string());
    assert_eq!(runs[2].0, author);
    assert_eq!(runs[2].1, vec!["add", "-A"]);
    assert_eq!(runs[3].0, author);
    assert_eq!(runs[3].1, vec!["status", "--porcelain"]);
    assert_eq!(runs[4].0, author);
    assert_eq!(
        runs[4].1,
        vec!["commit", "-m", "config: init [config/default]"]
    );
    assert_eq!(runs[5].0, repo);
    assert_eq!(runs[5].1[..3], ["worktree", "remove", "--force"]);
}

#[test]
fn scaffold_with_real_git_yields_exactly_one_config_ref() {
    // End to end with real git: the workspace has exactly the
    // config/default ref — no `main` (§2.2) — its head carries the
    // control files, and the authoring checkout is gone.
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("ws");
    let git = RealGit::new();
    scaffold(&dest, &crate::test_support::bare_roots(holder.path()), &git).unwrap();

    let repo = dest.join("repo.git");
    let refs = git
        .run_capture(&repo, &["for-each-ref", "--format=%(refname)"])
        .unwrap();
    assert_eq!(refs, "refs/heads/config/default");
    let providers = git
        .run_capture(&repo, &["show", "config/default:providers.yaml"])
        .unwrap();
    assert!(providers.contains("roles:"), "{providers}");
    assert!(!dest.join(".config-author").exists());
}

#[test]
fn scaffold_surfaces_descriptions_producer_failure() {
    // Malformed skill (no frontmatter) aborts the config-commit
    // authoring before anything is committed (§3.3): only the bare init
    // and the authoring-checkout creation ran.
    let holder = TempDir::new().unwrap();
    let data_root = holder.path().join("data");
    fs::create_dir_all(data_root.join("skills/broken")).unwrap();
    fs::write(data_root.join("skills/broken/SKILL.md"), "no frontmatter\n").unwrap();
    let git = StubGit::ok();
    let roots = crate::harness_root::Roots {
        config: holder.path().join("no-conf"),
        data: data_root,
    };
    let err = scaffold(&holder.path().join("ws"), &roots, &git).unwrap_err();
    assert!(matches!(err, ScaffoldError::Descriptions(_)), "got {err:?}");
    // init + worktree add, then the guard's teardown — nothing committed.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 3, "{runs:?}");
    assert_eq!(runs[2].1[..3], ["worktree", "remove", "--force"]);
    assert!(runs.iter().all(|(_, a)| a[0] != "commit"));
}

#[test]
fn scaffold_propagates_init_failure() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let err = scaffold(
        &dest,
        &crate::test_support::bare_roots(holder.path()),
        &StubGit::failing_at(0),
    )
    .unwrap_err();
    assert!(matches!(err, ScaffoldError::Git(_)), "got {err:?}");
}

#[test]
fn scaffold_propagates_each_git_failure_arm() {
    // Indexes: 0 init, 1 worktree add, 2 add -A, 3 status, 4 commit,
    // 5 worktree remove — each surfaces as ScaffoldError::Git. The last
    // is the teardown, which `Checkout::landed` reports on the success
    // path (only the failure paths swallow it).
    for idx in 1..=5 {
        let holder = TempDir::new().unwrap();
        let dest = holder.path().join("ws");
        let err = scaffold(
            &dest,
            &crate::test_support::bare_roots(holder.path()),
            &StubGit::failing_at(idx),
        )
        .unwrap_err();
        assert!(matches!(err, ScaffoldError::Git(_)), "idx {idx}: {err:?}");
    }
}

#[test]
fn scaffold_refuses_non_empty_dest() {
    let holder = TempDir::new().unwrap();
    fs::write(holder.path().join("x"), b"x").unwrap();
    let err = scaffold(
        holder.path(),
        &crate::test_support::bare_roots(holder.path()),
        &StubGit::ok(),
    )
    .unwrap_err();
    assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
}

#[test]
fn scaffold_surfaces_author_extract_io_error() {
    // A stub git whose worktree-add leaves a regular file squatting the
    // authoring checkout's path makes the extract step fail — the
    // post-init Io arm.
    struct SquattingGit;
    impl GitRunner for SquattingGit {
        fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
            if args.first() == Some(&"worktree") && args.get(1) == Some(&"add") {
                let author: &str = args.last().unwrap();
                let _ = fs::write(dest.parent().unwrap().join(".config-author"), b"squat");
                let _ = author;
            }
            Ok(())
        }
        fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
            self.run(dest, args).map(|_| String::new())
        }
    }
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("ws");
    let err = scaffold(
        &dest,
        &crate::test_support::bare_roots(holder.path()),
        &SquattingGit,
    )
    .unwrap_err();
    assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
}

#[test]
fn scaffold_surfaces_repo_dir_creation_failure() {
    // A regular file blocking <dest>/repo.git's creation exercises the
    // first Io arm (pre-git). `dest` itself must not exist for
    // check_dest, so block one level up: dest's parent is a file.
    let holder = TempDir::new().unwrap();
    let blocker = holder.path().join("blocker");
    fs::write(&blocker, b"file").unwrap();
    let err = scaffold(
        &blocker.join("ws"),
        &crate::test_support::bare_roots(holder.path()),
        &StubGit::ok(),
    )
    .unwrap_err();
    assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
}

#[test]
fn stub_run_capture_delegates_to_run() {
    let holder = TempDir::new().unwrap();
    assert_eq!(
        StubGit::ok().run_capture(holder.path(), &["x"]).unwrap(),
        ""
    );
    assert!(
        StubGit::failing_at(0)
            .run_capture(holder.path(), &["x"])
            .is_err()
    );
}
