//! **The conversation's spine** — the two reads under its history, and the one
//! act composed off them (bl-b52c).
//!
//! A fourth rows file, on the seam [`super::rows`]'s own doc draws three times
//! over. [`super::records`] is what an operator looks *under* a conversation
//! for; this is what its history is anchored TO — the operable commits, the
//! config commit governing them, and the fork whose `from` is one of the first.
//! The three are one subject and one file, exactly as the sign-in family's four
//! are: two of them are reads and one is an act, and splitting them to keep a
//! file tidy would split the fact.
//!
//! **Each answers a kind this seat now paints**, which is the admission test
//! every row here has passed since bl-213c: [`RAIL`] and [`GOVERNING`] answer
//! the two decoders this ball landed (`crate::reply::rail`,
//! `crate::reply::governing`), and [`fork`] answers a captured run, which is
//! the kind this seat has painted longest.
//!
//! # `fork` is a door and can never be a row
//!
//! [`super`]'s table is rows of **named strings**, and `fork` carries
//! `skills`, an array. The module doc says what happens then — *"a gesture
//! whose parameters are not all strings … is not added as a special case; it
//! goes through `ask` until there is a reframe that keeps this one table"* — so
//! it is a typed door with no row, on `effort`'s and `priority`'s own terms.
//! This seat composes the empty list and nothing else: a skill set is a
//! **choice** off the governing config, and the pane that would offer one does
//! not exist (bl-5c53).

use serde_json::{Value, json};

use crate::envelope;

use super::Verb;

/// **The spine.** Every operable commit the conversation has, and the children
/// dispatched off them.
pub const RAIL: Verb = Verb {
    word: "rail",
    params: &["workspace", "agent"],
    summary: "the conversation's spine: every operable commit and what hangs off it",
    detail: "One notch per step, each carrying the commit that step read \
             against, the spend as of it, and where in the chat its rule sits \
             — the points a conversation can be forked from. Beside them, the \
             children dispatched from this conversation: who each is, where it \
             forked from, what it is doing, what it has spent and the last \
             thing it said. A conversation nobody forked from answers notches \
             and no children, which is the honest empty case rather than an \
             error.",
};

/// **The governing config commit.**
pub const GOVERNING: Verb = Verb {
    word: "governing",
    params: &["workspace", "agent"],
    summary: "which config commit this conversation resolves its policy from",
    detail: "A conversation forks off a commit of a `config/*` lineage, and \
             the fork settles which lineage governs it — not which commit. \
             This answers the commit it resolves, short and full, which \
             lineage it follows, and every path that commit's tree holds. \
             Where lineages have diverged over its fork point there is no head \
             to follow, so it is held on that fork commit and the answer says \
             how many reached it. This is the BARE form; the wire also takes \
             `at` (resolve as of another commit), which this seat does not \
             compose — `lernie ask` carries it. It REFUSES rather than \
             answering absent: a conversation with no policy is never true.",
};

/// The word an act's control names, which has no row to read it off.
pub const FORK: &str = "fork";

/// The fields the door carries beyond the wall and the parent.
const PARENT: &str = "parent";
const FROM: &str = "from";
const ROLE: &str = "role";
const SKILLS: &str = "skills";
const GOAL: &str = "goal";

/// The spine, typed.
pub fn rail(workspace: String, agent: String) -> Value {
    RAIL.built(vec![workspace, agent])
}

/// The governing commit, typed. The bare form only — see [`GOVERNING`].
pub fn governing(workspace: String, agent: String) -> Value {
    GOVERNING.built(vec![workspace, agent])
}

/// **Fork `parent` from `from`, and give the child this goal.**
///
/// `from` is a **ref**: a commit off the conversation's own spine, or a
/// `config/<name>` head. Upstream is explicit that *"empty is not a value — a
/// seat refuses to fire without one, because a fork with no ref is a different
/// gesture"*, which is why the control that spends this door is offered on an
/// operable notch and nowhere else (`crate::reply::rail::Notch::operable`).
///
/// `role` is the model: litany resolves the provider and model id from that
/// name against `from`'s own governing config. So it is a name off a file this
/// seat cannot read yet, and the control says so rather than implying a
/// closed set (`crate::ui::records::spine`).
///
/// `skills` is spelled as the empty list, which is the wire's own way of
/// saying an attempt pins none.
pub fn fork(workspace: String, parent: String, from: String, role: String, goal: String) -> Value {
    json!({
        envelope::OP: FORK,
        envelope::WORKSPACE: workspace,
        PARENT: parent,
        FROM: from,
        ROLE: role,
        SKILLS: Vec::<String>::new(),
        GOAL: goal,
    })
}
