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
//! **`marks` is here as its READ.** With a branch it amends the wall's
//! tracking space, and that second form is a write with a confirmation to
//! design; the bare form reads, and the row below spells the bare form. The
//! reply is the same either way — *the branch re-read afterwards, never an
//! echo of what was asked* — so the seat's reading covers both the day the
//! write lands.

use serde_json::Value;

use super::Verb;

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

#[cfg(test)]
mod tests;
