//! **The balls family's reads** — the whole box's binding table, the fleet
//! board, one wall's own balls and the branch it tracks them on (yog's
//! `docs/REMOTE.md` §9.7; bl-d2af).
//!
//! Four rows and no doors, because all four take named strings or nothing:
//! two name no workspace at all and two name one. That is the only division
//! among them and it is the wire's, so it is read off the parameters rather
//! than listed here — [`super::Verb::addresses_a_workspace`] is the one home
//! of that predicate, and the asker fans the first two and aims the second two
//! off exactly it.
//!
//! # The five acts, and the `--as` name that is not an identity
//!
//! Three of them — [`ASSIGN`], [`RELEASE`], [`CLOSE`] — are three named
//! strings apiece and nothing else, so they are rows. The two authoring verbs
//! carry OPTIONAL text and are doors beside them ([`edit`]), on `effort`'s and
//! `fork`'s own terms.
//!
//! **`name` is the workspace's own name, never the operator's.** yog spells it
//! outright — *"the `--as` stamp every `bl` verb carries (§3.2): the ball's
//! bound workspace name, never the operator `$USER`"* — and its join binds a
//! ball to a workspace on exactly that equality. So this seat needs no
//! identity of its own to act: the stamp is the aimed wall's name as its
//! engine spells it (`crate::ui::Model::stamp`), and a seat that invented an
//! operator name would break the binding it was trying to make.
//!
//! **`marks` is here as its READ.** With a branch it amends the wall's
//! tracking space, and that second form is a write with a confirmation to
//! design; the bare form reads, and the row below spells the bare form. The
//! reply is the same either way — *the branch re-read afterwards, never an
//! echo of what was asked* — so the seat's reading covers both the day the
//! write lands.

use serde_json::Value;

use super::Verb;

/// The two authoring acts, whose text is optional and so cannot be rows.
pub mod edit;

/// **The fields the family's frames carry**, spelled once for the rows below
/// and for the doors in [`edit`] beside them — one home, so a row's `params`
/// and a door's `json!` can never disagree about what the wire calls a thing.
pub(crate) const PROJECT: &str = "project";
pub(crate) const ID: &str = "id";
pub(crate) const NAME: &str = "name";

/// The whole box's binding table.
pub const BALLS: Verb = Verb {
    word: "balls",
    params: &[],
    summary: "every ball⇄workspace binding fact",
    detail: "The join rows: which ball is claimed by which workspace, in which \
             state. It names no workspace, so its subject is EVERY channel this \
             box holds, and the pane is the union — the same fanned shape \
             `workspaces` and the decision queue keep. A ball nobody holds \
             names no claimant and no wall, which is a reading rather than a \
             blank.",
};

/// The fleet board.
pub const BOARD: Verb = Verb {
    word: "board",
    params: &[],
    summary: "the fleet board — every live ball in its column",
    detail: "The balls as columns: ready, gated, claimed, blocked. Gated is \
             balls' own close-blocker rule — a ball you could claim but could \
             not deliver — shown with the ball whose close mints its gate. \
             Each claimed row names the conversations working it and carries \
             what it has cost. It is also the ONLY answer carrying an armed \
             loop's own facts: there is no `fleet` read on this wire, and a \
             box running nothing leaves the array out rather than answering an \
             empty one.",
};

/// One wall's own balls.
pub const WORKSPACE_BALLS: Verb = Verb {
    word: "workspace-balls",
    params: &["workspace"],
    summary: "the balls one workspace holds, with what each has cost",
    detail: "Every ball bound to the named workspace: its id, its badge, the \
             project its `bl` verbs run in, the name they stamp `--as`, and \
             the tokens and money its conversations have spent on it. `balls` \
             answers the whole box's binding table; this answers one wall.",
};

/// The branch a wall tracks its tasks on.
pub const MARKS: Verb = Verb {
    word: "marks",
    params: &["workspace"],
    summary: "the branch this workspace tracks its tasks on",
    detail: "Each agent tracks on a balls branch of its own, in a task space \
             of its own, so two agents' task churn never collides. \
             `balls/tasks` is the project's shared board, which is where a \
             workspace raised to work an existing project is pointed. This is \
             the read; the form that amends the branch is a write this seat \
             does not compose.",
};

/// **Claim a ready ball for a workspace.**
pub const ASSIGN: Verb = Verb {
    word: "assign",
    params: &[PROJECT, ID, NAME],
    summary: "claim a ready ball for a workspace",
    detail: "Claims a ready ball (`bl claim`), which is what BINDS it to a \
             workspace: a bound ball is one a workspace holds, and the join is \
             on the name stamped here. So `name` is the claiming workspace's \
             own name as its engine spells it, never the operator's login. It \
             names no workspace field, so nothing routes it — the control that \
             fires it names the channel its row came down (DESIGN §4.35).",
};

/// **Let a held ball go.**
pub const RELEASE: Verb = Verb {
    word: "release",
    params: &[PROJECT, ID, NAME],
    summary: "unclaim a ball a workspace holds",
    detail: "Lets a ball go (`bl unclaim`): the workspace stops holding it and \
             anyone can claim it again. Nothing already committed in its \
             worktree is lost, which is what makes it the undoing of `assign` \
             rather than an unmaking of its own.",
};

/// **Deliver a held ball.**
pub const CLOSE: Verb = Verb {
    word: "close",
    params: &[PROJECT, ID, NAME],
    summary: "close a ball and deliver its work",
    detail: "Delivers a ball (`bl close`): folds `main` into its worktree, \
             runs the project's pre-commit gate, squashes the work onto the \
             target branch and removes the worktree. A failing gate aborts and \
             leaves the ball claimed. It is the one of the five with no verb \
             that undoes it, which is why the control that fires it is armed \
             (DESIGN §4.35).",
};

/// The binding table, typed.
pub fn balls() -> Value {
    BALLS.built(Vec::new())
}

/// The board, typed.
pub fn board() -> Value {
    BOARD.built(Vec::new())
}

/// One wall's balls, typed.
pub fn workspace_balls(workspace: String) -> Value {
    WORKSPACE_BALLS.built(vec![workspace])
}

/// One wall's tracking branch, typed.
pub fn marks(workspace: String) -> Value {
    MARKS.built(vec![workspace])
}

/// Claim it, typed.
pub fn assign(project: String, id: String, name: String) -> Value {
    ASSIGN.built(vec![project, id, name])
}

/// Let it go, typed.
pub fn release(project: String, id: String, name: String) -> Value {
    RELEASE.built(vec![project, id, name])
}

/// Deliver it, typed.
pub fn close(project: String, id: String, name: String) -> Value {
    CLOSE.built(vec![project, id, name])
}

#[cfg(test)]
mod tests;
