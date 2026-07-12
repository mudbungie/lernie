use super::*;
use tempfile::TempDir;

const BASH_SCHEMA: &str =
    r#"{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}"#;
const BASH_SKILL: &str =
    "---\nname: bash\ndescription: Run a shell command.\n---\n\n# bash\n\nbody text\n";

/// Lay out a data-root pool: `<root>/tools/*.json` and
/// `<root>/skills/<name>/SKILL.md`.
struct Pool {
    dir: TempDir,
}

impl Pool {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn tool(&self, name: &str, schema: &str) -> &Self {
        let d = self.root().join(TOOLS_SUBDIR);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(format!("{name}.json")), schema).unwrap();
        self
    }
    fn skill(&self, name: &str, skill_md: &str) -> &Self {
        let d = self.root().join(SKILLS_SUBDIR).join(name);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(SKILL_MANIFEST), skill_md).unwrap();
        self
    }
}

fn worktree() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn copies_tool_schema_verbatim() {
    let pool = Pool::new();
    pool.tool("bash", BASH_SCHEMA);
    let wt = worktree();
    snapshot(pool.root(), wt.path()).unwrap();

    let out = wt.path().join("descriptions/tools/bash.json");
    assert_eq!(fs::read_to_string(&out).unwrap(), BASH_SCHEMA);
}

#[test]
fn writes_skill_frontmatter_body_only() {
    let pool = Pool::new();
    pool.skill("bash", BASH_SKILL);
    let wt = worktree();
    snapshot(pool.root(), wt.path()).unwrap();

    let out = wt.path().join("descriptions/skills/bash.md");
    let body = fs::read_to_string(&out).unwrap();
    // The fenced markdown body is stripped; only the frontmatter YAML
    // survives, verbatim, and round-trips through the shared parser.
    assert_eq!(body, "name: bash\ndescription: Run a shell command.\n");
    let fm = crate::skill::parse(&body).unwrap();
    assert_eq!(fm.description, "Run a shell command.");
}

#[test]
fn absent_pools_yield_empty_descriptions_tree() {
    // §3.3: empty (here: entirely absent) pool → no descriptions tree.
    let wt = worktree();
    snapshot(Path::new("/no/such/data/root"), wt.path()).unwrap();
    assert!(!wt.path().join("descriptions").exists());
}

#[test]
fn empty_pool_dirs_create_no_descriptions_subdirs() {
    let pool = Pool::new();
    fs::create_dir_all(pool.root().join(TOOLS_SUBDIR)).unwrap();
    fs::create_dir_all(pool.root().join(SKILLS_SUBDIR)).unwrap();
    let wt = worktree();
    snapshot(pool.root(), wt.path()).unwrap();
    assert!(!wt.path().join("descriptions/tools").exists());
    assert!(!wt.path().join("descriptions/skills").exists());
}

#[test]
fn non_json_pool_files_and_non_dir_skill_entries_are_skipped() {
    let pool = Pool::new();
    pool.tool("bash", BASH_SCHEMA);
    // A stray non-.json file in the tools pool, a directory named like a
    // schema (not a file), and a stray file in the skills pool are all
    // ignored.
    fs::write(pool.root().join(TOOLS_SUBDIR).join("README.md"), b"x").unwrap();
    fs::create_dir_all(pool.root().join(TOOLS_SUBDIR).join("notafile.json")).unwrap();
    fs::create_dir_all(pool.root().join(SKILLS_SUBDIR)).unwrap();
    fs::write(pool.root().join(SKILLS_SUBDIR).join("loose.txt"), b"x").unwrap();
    let wt = worktree();
    snapshot(pool.root(), wt.path()).unwrap();

    assert!(wt.path().join("descriptions/tools/bash.json").is_file());
    assert!(!wt.path().join("descriptions/tools/README.md").exists());
    assert!(!wt.path().join("descriptions/tools/notafile.json").exists());
    assert!(!wt.path().join("descriptions/skills").exists());
}

#[test]
fn skill_dir_without_manifest_is_not_an_available_skill() {
    let pool = Pool::new();
    // Directory present, but no SKILL.md inside.
    fs::create_dir_all(pool.root().join(SKILLS_SUBDIR).join("empty-skill")).unwrap();
    let wt = worktree();
    snapshot(pool.root(), wt.path()).unwrap();
    assert!(!wt.path().join("descriptions/skills").exists());
}

#[test]
fn skill_manifest_without_frontmatter_is_a_loud_error() {
    let pool = Pool::new();
    pool.skill("broken", "no frontmatter here\n");
    let wt = worktree();
    let err = snapshot(pool.root(), wt.path()).unwrap_err();
    match err {
        Error::NoFrontmatter { name } => assert_eq!(name, "broken"),
        other => panic!("expected NoFrontmatter, got {other:?}"),
    }
    // The error renders with the SKILL.md filename for the operator.
    assert!(
        snapshot(pool.root(), wt.path())
            .unwrap_err()
            .to_string()
            .contains("SKILL.md"),
    );
}

#[test]
fn snapshot_is_deterministic_across_many_artifacts() {
    let pool = Pool::new();
    pool.tool("bash", BASH_SCHEMA)
        .tool("read_file", r#"{"type":"object"}"#);
    pool.skill("bash", BASH_SKILL).skill(
        "read_file",
        "---\nname: read_file\ndescription: Read a file.\n---\n",
    );
    let wt = worktree();
    snapshot(pool.root(), wt.path()).unwrap();

    for name in ["bash", "read_file"] {
        assert!(
            wt.path()
                .join(format!("descriptions/tools/{name}.json"))
                .is_file()
        );
        assert!(
            wt.path()
                .join(format!("descriptions/skills/{name}.md"))
                .is_file()
        );
    }
}

#[test]
fn tool_copy_failure_surfaces_io_error() {
    let pool = Pool::new();
    pool.tool("bash", BASH_SCHEMA);
    let wt = worktree();
    // Plant a *directory* where the schema file must be written, so
    // `fs::copy` fails on the destination.
    fs::create_dir_all(wt.path().join("descriptions/tools/bash.json")).unwrap();
    let err = snapshot(pool.root(), wt.path()).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "got {err:?}");
}

#[test]
fn tool_dest_dir_creation_failure_surfaces_io_error() {
    let pool = Pool::new();
    pool.tool("bash", BASH_SCHEMA);
    let wt = worktree();
    // A regular file at `descriptions/` makes `create_dir_all` of the
    // tools subdir fail (a parent component is not a directory).
    fs::write(wt.path().join("descriptions"), b"blocker").unwrap();
    let err = snapshot(pool.root(), wt.path()).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "got {err:?}");
}

#[test]
fn skill_manifest_read_failure_surfaces_io_error() {
    let pool = Pool::new();
    // A *directory* named SKILL.md makes `read_to_string` fail with a
    // non-NotFound error, surfaced rather than skipped.
    let d = pool.root().join(SKILLS_SUBDIR).join("bad");
    fs::create_dir_all(d.join(SKILL_MANIFEST)).unwrap();
    let wt = worktree();
    let err = snapshot(pool.root(), wt.path()).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "got {err:?}");
}

#[test]
fn skill_dest_dir_creation_failure_surfaces_io_error() {
    let pool = Pool::new();
    pool.skill("bash", BASH_SKILL);
    let wt = worktree();
    // A regular file at `descriptions/` blocks creating skills/ under it.
    fs::write(wt.path().join("descriptions"), b"blocker").unwrap();
    let err = snapshot(pool.root(), wt.path()).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "got {err:?}");
}

#[test]
fn skill_write_failure_surfaces_io_error() {
    let pool = Pool::new();
    pool.skill("bash", BASH_SKILL);
    let wt = worktree();
    // A directory where the .md file must go makes `fs::write` fail.
    fs::create_dir_all(wt.path().join("descriptions/skills/bash.md")).unwrap();
    let err = snapshot(pool.root(), wt.path()).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "got {err:?}");
}

#[test]
fn unreadable_pool_dir_surfaces_io_error() {
    // A regular file where the tools *pool* is expected: read_dir fails
    // with a non-NotFound error, surfaced (not treated as empty).
    let holder = TempDir::new().unwrap();
    let data_root = holder.path().join("root");
    fs::create_dir_all(&data_root).unwrap();
    fs::write(data_root.join(TOOLS_SUBDIR), b"not a dir").unwrap();
    let wt = worktree();
    let err = snapshot(&data_root, wt.path()).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "got {err:?}");
}
