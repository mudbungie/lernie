//! Coverage for excluding the agent `name` from the transfer (bl-c8ed).
//!
//! `name` is branch-scoped context (ARCH §2.2, §2.3): every dispatch
//! commit rewrites it, so a child forked off a *named* parent carries a
//! rewrite of that blob in its fork-point→terminal diff. Without the
//! exclusion the diff would carry the child's name — usually the empty
//! one of an unnamed child — back over its parent's and unname it, which
//! is the failure the "always write, never delete" shape (§2.3) exists
//! together with this exclusion to make impossible.

use super::super::apply;
use super::{git, init_repo, make_child, write};
use crate::template::GitRunner;

#[test]
fn apply_never_carries_a_childs_name_back_over_its_parents() {
    let dir = init_repo();
    let wt = dir.path();
    let g = git();
    // The parent wears a name, so the fork point carries the blob.
    write(wt, "name", "pale-otter\n");
    g.run(wt, &["add", "-A"]).unwrap();
    g.run(wt, &["commit", "-m", "settle name"]).unwrap();

    // The child's own dispatch commit rewrites it — empty, the unnamed
    // shape — and the child also does real work worth transferring.
    let terminal = make_child(wt, &[("name", ""), ("feature.txt", "f\n")]);
    apply(wt, "p-child", &terminal, &git()).unwrap();

    assert_eq!(
        std::fs::read_to_string(wt.join("feature.txt")).unwrap(),
        "f\n",
        "the work product still transfers"
    );
    assert_eq!(
        std::fs::read_to_string(wt.join("name")).unwrap(),
        "pale-otter\n",
        "a child's name never reaches its parent's tree (§2.6)"
    );
}
