//! The public-declaration extractor: every externally reachable
//! declaration *of one module*, rendered as a `"<kind> <name>"` string.
//!
//! Whether the module itself is reachable is [`crate::graph`]'s verdict;
//! this decides what, inside it, is public. It is deliberately member-
//! deep — a leak is far likelier to be a new `pub` field, a `pub` method
//! on an `impl` block, or a new enum variant than a new top-level item:
//!
//! - a type's `pub` fields, its variants, and the traits it `derive`s or
//!   `impl`s (derived and implemented behaviour is public behaviour);
//! - the `pub` members of its inherent `impl` blocks — invisible to any
//!   top-level-item scan, and the surface's widest blind spot before;
//! - `#[macro_export]`ed macros, which land at the crate root regardless
//!   of the module's own publicity.
//!
//! Every `syn::Item` variant that can carry `pub` on stable Rust is
//! classified — the catch-all arm sees only non-public items.
//! [`extractor_classifies_every_public_shape`] pins that totality against
//! a synthetic module carrying every shape at once, both the ones it must
//! record and the ones it must ignore, so the extractor cannot silently
//! stop recognising a shape.

use syn::punctuated::Punctuated;
use syn::{Fields, ImplItem, Item, Token, UseTree, Visibility};

/// Externally-public only: `pub` proper, never `pub(crate)`/`pub(super)`
/// (those are `Visibility::Restricted` and unreachable outside the crate).
pub fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// A path rendered as written, ignoring generic arguments:
/// `std::fmt::Display`, `From`.
fn path_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// The bare name of an `impl` block's self type.
fn self_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .expect("impl self type has a segment")
            .ident
            .to_string(),
        _ => panic!("unclassified `impl` self type — extend the extractor"),
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

/// The traits a type derives — public behaviour, so part of the surface
/// (and the evidence that a verb's `Args` is the very type clap parses).
fn derives(name: &str, attrs: &[syn::Attribute], out: &mut Vec<String>) {
    for attr in attrs.iter().filter(|a| a.path().is_ident("derive")) {
        let list = attr
            .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
            .expect("derive list parses");
        for path in list {
            out.push(format!("derive {name}: {}", path_name(&path)));
        }
    }
}

/// A type's `pub` fields, by name (or by index, for a tuple type). A
/// variant's fields carry no visibility of their own — they are as public
/// as the enum — so this records nothing for them; the payload *types*
/// are pinned by [`crate::entries`].
fn fields(name: &str, fields: &Fields, out: &mut Vec<String>) {
    for (index, f) in fields.iter().enumerate() {
        if is_pub(&f.vis) {
            let field = f
                .ident
                .as_ref()
                .map_or_else(|| index.to_string(), ToString::to_string);
            out.push(format!("field {name}.{field}"));
        }
    }
}

/// An `impl` block: a trait implementation is one public fact; an
/// inherent block contributes each of its `pub` members.
fn impl_block(block: &syn::ItemImpl, out: &mut Vec<String>) {
    let ty = self_name(&block.self_ty);
    if let Some((_, path, _)) = &block.trait_ {
        out.push(format!("impl {ty}: {}", path_name(path)));
        return;
    }
    for item in &block.items {
        match item {
            ImplItem::Fn(f) if is_pub(&f.vis) => out.push(format!("method {ty}::{}", f.sig.ident)),
            ImplItem::Const(c) if is_pub(&c.vis) => {
                out.push(format!("assoc-const {ty}::{}", c.ident));
            }
            ImplItem::Type(t) if is_pub(&t.vis) => {
                out.push(format!("assoc-type {ty}::{}", t.ident))
            }
            _ => {}
        }
    }
}

/// The `#[macro_export]`ed macros of a module. They land at the crate
/// root whatever their module's own visibility, so they are checked in
/// every module, not only the reachable ones.
pub fn exported_macros(items: &[Item]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Macro(m) if m.attrs.iter().any(|a| a.path().is_ident("macro_export")) => {
                Some(format!(
                    "macro {}",
                    m.ident.as_ref().expect("an exported macro is named")
                ))
            }
            _ => None,
        })
        .collect()
}

/// Every externally reachable declaration of a module's items.
pub fn entries(items: &[Item]) -> Vec<String> {
    let mut out = exported_macros(items);
    for item in items {
        match item {
            Item::Mod(m) if is_pub(&m.vis) => out.push(format!("mod {}", m.ident)),
            Item::Struct(s) if is_pub(&s.vis) => {
                let name = s.ident.to_string();
                out.push(format!("struct {name}"));
                derives(&name, &s.attrs, &mut out);
                fields(&name, &s.fields, &mut out);
            }
            Item::Union(u) if is_pub(&u.vis) => {
                let name = u.ident.to_string();
                out.push(format!("union {name}"));
                derives(&name, &u.attrs, &mut out);
                fields(&name, &Fields::Named(u.fields.clone()), &mut out);
            }
            Item::Enum(e) if is_pub(&e.vis) => {
                let name = e.ident.to_string();
                out.push(format!("enum {name}"));
                derives(&name, &e.attrs, &mut out);
                for v in &e.variants {
                    out.push(format!("variant {name}::{}", v.ident));
                }
            }
            Item::Fn(f) if is_pub(&f.vis) => out.push(format!("fn {}", f.sig.ident)),
            Item::Const(c) if is_pub(&c.vis) => out.push(format!("const {}", c.ident)),
            Item::Static(s) if is_pub(&s.vis) => out.push(format!("static {}", s.ident)),
            Item::Type(t) if is_pub(&t.vis) => out.push(format!("type {}", t.ident)),
            Item::Trait(t) if is_pub(&t.vis) => out.push(format!("trait {}", t.ident)),
            Item::ExternCrate(c) if is_pub(&c.vis) => out.push(format!("extern-crate {}", c.ident)),
            Item::Use(u) if is_pub(&u.vis) => {
                out.extend(use_leaves(&u.tree).into_iter().map(|n| format!("use {n}")));
            }
            Item::Impl(i) => impl_block(i, &mut out),
            _ => {}
        }
    }
    out
}

// ── The extractor is total over every public shape ──────────────────────

/// Every shape the extractor must record and — in the same source — the
/// non-`pub`, `pub(crate)` and inherent-visibility shapes it must ignore.
/// Set equality proves both directions at once.
#[test]
fn extractor_classifies_every_public_shape() {
    let src = "\
        pub mod m; pub struct S { pub f: u8, private: u8 } pub enum E { V, W(u8) } \
        pub fn f() {} pub const C: u8 = 0; pub static ST: u8 = 0; pub type T = u8; \
        pub trait Tr { type Assoc; } pub union U { pub bits: u8 } pub extern crate other; \
        pub use a::b::name; pub use a::{g, r as R, *}; \
        #[derive(Debug, clap::Args)] pub struct D(pub u8, u8); \
        impl S { pub fn method(&self) {} pub const AC: u8 = 0; pub type At = u8; fn hidden() {} } \
        impl Tr for S { type Assoc = u8; } \
        fn private() {} pub(crate) fn restricted() {} struct Hidden { pub f: u8 } \
        mod inner { pub fn unreachable() {} } \
        #[macro_export] macro_rules! exported { () => {} } \
        macro_rules! unexported { () => {} }";
    let got: std::collections::BTreeSet<String> = entries(&syn::parse_file(src).unwrap().items)
        .into_iter()
        .collect();
    let want: std::collections::BTreeSet<String> = [
        "mod m",
        "struct S",
        "field S.f",
        "enum E",
        "variant E::V",
        "variant E::W",
        "fn f",
        "const C",
        "static ST",
        "type T",
        "trait Tr",
        "union U",
        "field U.bits",
        "extern-crate other",
        "use name",
        "use g",
        "use R",
        "use *",
        "struct D",
        "derive D: Debug",
        "derive D: clap::Args",
        "field D.0",
        "method S::method",
        "assoc-const S::AC",
        "assoc-type S::At",
        "impl S: Tr",
        "macro exported",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(got, want, "public-declaration extractor shape drift");
}

/// An `impl` self type the extractor cannot name is rejected, never
/// silently skipped — an unrecognised shape must fail loudly.
#[test]
#[should_panic(expected = "unclassified `impl` self type")]
fn a_non_path_impl_self_type_is_rejected() {
    entries(
        &syn::parse_file("impl [u8] { pub fn f() {} }")
            .unwrap()
            .items,
    );
}
