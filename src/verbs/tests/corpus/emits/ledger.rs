//! **The frames the seat's encoder cannot compose**, by op, count and reason —
//! rule 3 of the corpus contract, written down rather than passed over.
//!
//! Split from [`super`] at the 300-line cap on the seam the two already have:
//! [`super`] is the MECHANISM — one frame in, this seat's own encoding out —
//! and this is the LEDGER of decisions that mechanism could not make. The
//! first moves when a builder changes; the second only when somebody decides a
//! surface is not worth building, or builds it (bl-4371 built the path rung
//! and bl-9fd1 the cascade, and this file is where each is visible).

/// **The frames the seat's encoder cannot compose**, by op, count and reason.
///
/// Every entry is a surface this build does not have rather than a field it
/// drops. A count that moves — because yog grew a rung, or because a pane
/// landed here — fails until the reason is rewritten, which is the whole point
/// of writing it down.
pub(super) const UNEMITTED: &[(&str, usize, &str)] = &[
    (
        "files",
        3,
        "the `at` and `path` forms: this seat composes the bare listing only \
         — pinning a commit and previewing one file are controls the records \
         pane does not have yet, and a seat that guessed either would answer \
         a question nobody asked",
    ),
    (
        "marks",
        2,
        "the amending form: this seat composes the bare READ only, and pointing \
         a wall's task space at another branch is a write with a confirmation \
         to design — the ball pane's five acts landed in bl-f7ae and this one \
         did not, because a tracking branch is not a ball",
    ),
    (
        "create",
        1,
        "the scheduling fields: this seat composes the title and the body, and \
         a priority, a tag, a parent and a blocker are four pickers the ball \
         pane does not have — `fields` is an array of objects besides, which is \
         why the door composes text and nothing else (`crate::verbs::balls::\
         edit`)",
    ),
    (
        "update",
        1,
        "the scheduling fields, on the filing's own terms: this seat amends a \
         ball's text and appends to its journal, and the four facts the board \
         orders on are the same four pickers",
    ),
    (
        "work-diff",
        2,
        "the two file forms: this seat composes the bare listing only — naming \
         a ball and a path answers that one file's PATCH instead, and a \
         control that guessed at a file would answer a question nobody asked \
         (`crate::verbs::fleet`)",
    ),
    (
        "prepare",
        8,
        "the ball rung: this seat composes the bare rung and the PATH rung \
         (bl-4371, `lernie start <workspace> <goal> [<dir>]`), and the ball \
         rung's eight join states need a project, a picker and a claim — a \
         seat that guessed one would found a claim nobody asked for",
    ),
    (
        "governing",
        1,
        "the `at` form: this seat composes the bare read only — resolving the \
         policy as of another commit is the pin the records pane does not \
         have, and a seat that guessed a rev would answer about a tree \
         nobody named",
    ),
    (
        "fork",
        1,
        "the skill list: this seat composes an attempt that pins none, because \
         the skills a lineage declares are a read the config-file pane owns \
         and this window has no way to offer them",
    ),
    (
        "fan",
        1,
        "the ball-less form: upstream reads it as the ENGINE's own focused \
         ball, and a seat has no focus at the far end — every gesture this \
         window composes names the ball off the work-diff row it fired from \
         (`crate::verbs::candidates`)",
    ),
    (
        "deliver",
        1,
        "the ball-less form, on the fan's own terms: the obligation comes off \
         the row, never off a focus this seat cannot see",
    ),
    (
        "retire",
        1,
        "the ball-less form, on the fan's own terms: the obligation comes off \
         the row, never off a focus this seat cannot see",
    ),
    (
        "prompt",
        6,
        "a predicted conversation seed: this seat spells `seed` null, because \
         the mint is the engine's and a seat that predicted one would have to \
         fire the name it painted",
    ),
];
