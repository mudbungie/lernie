//! The command-surface parity checker (ARCH §3.4, CI-enforced both
//! directions).
//!
//! "The crate exposes nothing public that is not a verb's entry, its
//! arguments, its products, or the binding preludes … and no verb lacks
//! its entry." This test is the enforcement mechanism for that invariant.
//! It rides ordinary `cargo test` → tarpaulin → `make check` → the
//! pre-commit hook and GitHub Actions, with no new toolchain.
//!
//! It is itself a consumer of the public surface — it may link nothing but
//! [`lernie::cmd`]. The ground truth of "what is `pub`" is the crate's own
//! source, parsed with `syn`; the ground truth of "what is a verb" is the
//! CLI's introspected subcommand set, read from clap at runtime. A
//! bijection between the two is the invariant.

use std::collections::BTreeSet;
use std::path::PathBuf;
use syn::{Item, UseTree, Visibility};

/// A crate-root-relative source file's parsed AST.
fn parse_src(rel: &str) -> syn::File {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

/// Externally-public only: `pub` proper, never `pub(crate)`/`pub(super)`
/// (those are `Visibility::Restricted` and unreachable outside the crate).
fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// One externally-public top-level item: its `(kind, name)`. `pub use`
/// re-exports are resolved to their leaf names — they count as public
/// items of the re-exporting module (§3.4: the preludes are re-exports).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PubItem {
    kind: &'static str,
    name: String,
}

impl PubItem {
    fn new(kind: &'static str, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }
}

/// Flatten a `use` tree to the names it brings into scope.
fn use_leaves(tree: &UseTree) -> Vec<String> {
    match tree {
        UseTree::Path(p) => use_leaves(&p.tree),
        UseTree::Name(n) => vec![n.ident.to_string()],
        UseTree::Rename(r) => vec![r.rename.to_string()],
        UseTree::Glob(_) => vec!["*".to_string()],
        UseTree::Group(g) => g.items.iter().flat_map(use_leaves).collect(),
    }
}

/// Every externally-public top-level item of a parsed source file.
fn public_items(file: &syn::File) -> Vec<PubItem> {
    let mut out = Vec::new();
    for item in &file.items {
        match item {
            Item::Mod(m) if is_pub(&m.vis) => out.push(PubItem::new("mod", m.ident.to_string())),
            Item::Struct(s) if is_pub(&s.vis) => {
                out.push(PubItem::new("struct", s.ident.to_string()))
            }
            Item::Enum(e) if is_pub(&e.vis) => out.push(PubItem::new("enum", e.ident.to_string())),
            Item::Fn(f) if is_pub(&f.vis) => out.push(PubItem::new("fn", f.sig.ident.to_string())),
            Item::Const(c) if is_pub(&c.vis) => {
                out.push(PubItem::new("const", c.ident.to_string()))
            }
            Item::Static(s) if is_pub(&s.vis) => {
                out.push(PubItem::new("static", s.ident.to_string()))
            }
            Item::Type(t) if is_pub(&t.vis) => out.push(PubItem::new("type", t.ident.to_string())),
            Item::Trait(t) if is_pub(&t.vis) => {
                out.push(PubItem::new("trait", t.ident.to_string()))
            }
            Item::Use(u) if is_pub(&u.vis) => {
                for name in use_leaves(&u.tree) {
                    out.push(PubItem::new("use", name));
                }
            }
            _ => {}
        }
    }
    out
}

/// The public `mod`s declared in `src/cmd/mod.rs` (verb modules + prelude).
fn cmd_public_mods() -> BTreeSet<String> {
    public_items(&parse_src("src/cmd/mod.rs"))
        .into_iter()
        .filter(|i| i.kind == "mod")
        .map(|i| i.name)
        .collect()
}

/// The verb modules: the public `mod`s of `cmd` minus the `prelude` seam.
fn verb_modules() -> BTreeSet<String> {
    let mut mods = cmd_public_mods();
    mods.remove("prelude");
    mods
}

/// The subcommand names clap reports for the shared `Cli` at runtime.
fn cli_subcommands() -> BTreeSet<String> {
    use clap::CommandFactory;
    lernie::cmd::Cli::command()
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect()
}

// ── (a) The crate root exposes only `cmd` ───────────────────────────────

#[test]
fn lib_root_exposes_exactly_pub_mod_cmd() {
    let items = public_items(&parse_src("src/lib.rs"));
    assert_eq!(
        items,
        vec![PubItem::new("mod", "cmd")],
        "src/lib.rs must expose exactly one public item, `pub mod cmd`; found {items:?}",
    );
}

// ── (b) Verb modules ↔ CLI subcommands, both inclusions ─────────────────

#[test]
fn every_cli_verb_has_a_library_entry_module() {
    let mods = verb_modules();
    for verb in cli_subcommands() {
        assert!(
            mods.contains(&verb),
            "verb {verb:?} has no library entry module (no public `mod {verb}` in src/cmd)",
        );
    }
}

#[test]
fn every_public_cmd_module_is_a_cli_verb() {
    let subs = cli_subcommands();
    for module in verb_modules() {
        assert!(
            subs.contains(&module),
            "public module {module:?} is not a CLI verb (cmd::{module} has no subcommand)",
        );
    }
}

// ── (c) Each verb module exposes exactly {Args, run} ────────────────────

#[test]
fn each_verb_module_exposes_exactly_args_and_run() {
    let want: BTreeSet<PubItem> = [PubItem::new("struct", "Args"), PubItem::new("fn", "run")]
        .into_iter()
        .collect();
    for verb in verb_modules() {
        let items = public_items(&parse_src(&format!("src/cmd/{verb}.rs")));
        let got: BTreeSet<PubItem> = items.iter().cloned().collect();
        assert_eq!(
            got, want,
            "verb module cmd::{verb} must expose exactly {{Args (struct), run (fn)}}; found {items:?}",
        );
    }
}

// ── (d) The cmd module + prelude expose exactly the binding seam ────────

#[test]
fn cmd_module_public_surface_is_exactly_the_binding_seam() {
    let items = public_items(&parse_src("src/cmd/mod.rs"));
    let got: BTreeSet<PubItem> = items.iter().cloned().collect();
    let mut want: BTreeSet<PubItem> = [
        PubItem::new("struct", "Cli"),
        PubItem::new("struct", "Fx"),
        PubItem::new("struct", "Error"),
        PubItem::new("enum", "Command"),
        PubItem::new("enum", "Outcome"),
    ]
    .into_iter()
    .collect();
    // Plus the verb modules and the prelude seam — the public `mod`s.
    for m in cmd_public_mods() {
        want.insert(PubItem::new("mod", m));
    }
    assert_eq!(
        got, want,
        "cmd's public surface must be exactly {{Cli, Command, Outcome, Fx, Error, verb mods, prelude}}; found {items:?}",
    );
}

#[test]
fn prelude_exposes_exactly_the_three_binding_mechanisms() {
    let items = public_items(&parse_src("src/cmd/prelude.rs"));
    let got: BTreeSet<String> = items.iter().map(|i| i.name.clone()).collect();
    let want: BTreeSet<String> = ["become_pgid_leader", "install_stop_handler", "stop_flag"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        got, want,
        "cmd::prelude must re-export exactly the three binding preludes; found {items:?}",
    );
}

// ── (e) Each Command variant wires its own verb module's Args ───────────

/// The module segment of a `<module>::Args` variant field type, asserting
/// the type is exactly a two-or-more-segment path ending in `Args`.
fn args_module_segment(variant: &str, ty: &syn::Type) -> String {
    let syn::Type::Path(tp) = ty else {
        panic!("Command::{variant} field type is not a path (expected `<module>::Args`)");
    };
    let segs = &tp.path.segments;
    let joined: Vec<String> = segs.iter().map(|s| s.ident.to_string()).collect();
    assert!(
        segs.len() >= 2 && joined.last().map(String::as_str) == Some("Args"),
        "Command::{variant} field type must be `<module>::Args`; found `{}`",
        joined.join("::"),
    );
    joined[joined.len() - 2].clone()
}

/// `(variant, module-of-its-Args)` for every `Command` variant.
fn command_variants() -> Vec<(String, String)> {
    let file = parse_src("src/cmd/mod.rs");
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Enum(e) = item else { continue };
        if e.ident != "Command" {
            continue;
        }
        for v in &e.variants {
            let variant = v.ident.to_string();
            let syn::Fields::Unnamed(f) = &v.fields else {
                panic!(
                    "Command::{variant} must be a single-field tuple variant `Variant(<module>::Args)`"
                );
            };
            assert_eq!(
                f.unnamed.len(),
                1,
                "Command::{variant} must carry exactly one field (its verb's Args)",
            );
            let module = args_module_segment(&variant, &f.unnamed[0].ty);
            out.push((variant, module));
        }
    }
    assert!(!out.is_empty(), "Command enum not found in src/cmd/mod.rs");
    out
}

#[test]
fn each_command_variant_wires_its_own_verb_module() {
    let verbs = verb_modules();
    for (variant, module) in command_variants() {
        let expected = variant.to_lowercase();
        assert_eq!(
            module, expected,
            "Command::{variant} wires `{module}::Args`, but its verb module is `{expected}` (mispaired variant→module)",
        );
        assert!(
            verbs.contains(&module),
            "Command::{variant} names module `{module}`, which is not a verb module",
        );
    }
}

// ── Robustness: the extractors handle every item/use shape ──────────────
// The real sources use only some shapes; parse a synthetic module so the
// extractors stay correct (and covered) for the shapes they might one day
// meet — a stray `pub const`/`pub type`/`pub use … as …` must not slip past.

#[test]
fn extractors_cover_every_public_item_and_use_shape() {
    // Every public item/use shape, plus non-`pub`/`pub(crate)`/`impl` items
    // that must be excluded — set equality proves both directions at once.
    let src = "pub mod m; pub struct S; pub enum E {} pub fn f() {} \
               pub const C: u8 = 0; pub static ST: u8 = 0; pub type T = u8; \
               pub trait Tr {} pub use a::b::name; pub use a::{g, r as R, *}; \
               fn private() {} pub(crate) fn restricted() {} impl S {}";
    let got: BTreeSet<String> = public_items(&syn::parse_file(src).unwrap())
        .iter()
        .map(|i| format!("{}:{}", i.kind, i.name))
        .collect();
    let want = "mod:m struct:S enum:E fn:f const:C static:ST type:T trait:Tr \
                use:name use:g use:R use:*";
    let want: BTreeSet<String> = want.split_whitespace().map(String::from).collect();
    assert_eq!(got, want, "public-item/use extractor shape drift");
}
