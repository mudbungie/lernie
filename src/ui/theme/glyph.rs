//! **The glyph an act wears** (bl-f251) — an icon is a token exactly as a
//! colour is: a pane asks for one by the name of the act, and this is the one
//! place the character is spelled. Three are the plainest sign for what the
//! act does to the turn: a bolt cuts in, a play mark advances, a square stops.
//! The fourth is not an act on a turn at all — it is the composer's other
//! mode, where the row carries the start's own parameter (DESIGN §4.39), and a
//! pencil is the plainest sign for *drafted rather than done*. Every one is
//! asserted to have a glyph in the body font (`super::tests`), because a tofu
//! box is a control with no word on it.
//!
//! Split from [`super`] at the cap when the bubbles arrived (§4.45 item 7):
//! the glyphs are the one table there that is not a colour.

pub const INTERRUPT: &str = "⚡";
pub const NUDGE: &str = "▶";
pub const STOP: &str = "■";
pub const PLAN: &str = "✏";
/// **The prompt** (§4.42): the glyph at the left edge of the line the
/// operator types into, in the brand.
pub const PROMPT: &str = "›";
/// **The rule's lead** (`crate::ui::status`), at both of its ends. An em
/// dash where the baseline draws a box-drawing line: the body font has
/// no `─`.
pub const LEAD: &str = "—";
/// **What parts the rule's facts** — a bar, for the same reason: no `│`.
pub const SEP: &str = "|";
/// **The model's gutter** (`crate::ui::chat`): a broken bar where the
/// baseline draws `┊`, which the body font has not.
pub const MODEL: &str = "¦";
/// **A note's gutter**: a peer, a tool's answer, the machinery's own word.
pub const NOTE: &str = "·";
/// **A call on the trail.**
pub const CALL: &str = "•";
/// **A call that returned**, and one that failed.
pub const DONE: &str = "✔";
pub const FAILED: &str = "✖";
/// **A fold, open and shut**: the minus and the plus, because the body
/// font has no small triangles.
pub const OPEN: &str = "−";
pub const SHUT: &str = "+";
/// **A conversation's state in the switcher** (§4.42 item 7).
pub const WORKING: &str = "▶";
pub const ASKING: &str = "?";
pub const AT_REST: &str = "✔";
pub const STOPPED: &str = "■";
/// **Where a line was cut** — a call's head on the trail.
pub const ELIDED: &str = "…";
/// **A selector on the rule** (§4.42 item 11): it opens upward. The body
/// font has no `▾` or `▴`; it has this.
pub const DROP: &str = "⏶";
/// **A dropup's refresh in flight**, in the annotation accent — the
/// palette has no yellow (`docs/STYLE.md` §1).
pub const RELOAD: &str = "↻";
/// **A tab's close** (DESIGN §4.44 item 2), in faint ink.
pub const CLOSE: &str = "×";
/// **The bar's pin** (DESIGN §4.44), alone at the collapsed head.
pub const PIN: &str = "📌";
/// **A row's trashcan** (DESIGN §4.44 item 1; bl-e837), at its right end
/// while the pointer is over it, in faint ink.
pub const TRASH: &str = "🗑";
