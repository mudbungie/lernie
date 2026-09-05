+++
title = "a prepared start cannot be fanned, and no candidate can be accepted or retired"
created = 1788397354
updated = 1788583562
claimant = "Animations-R"
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Three `control`-classed ops with no interactable here: `fan`, `deliver`, `retire`.

The seat can prepare a start and fire it (`act:prepare act:prompt` on the start control), which is the single-attempt path. The n-candidate path is unreachable: nothing spreads a prepared start over isolated attempts, nothing accepts one of the candidates that come back, and nothing releases a candidate's worktree afterwards. Accepting one of n is an operator judgement, which is why yog classes `deliver` `control` rather than machine (docs/PARITY.md §10).

Exemption rows in `parity.toml` cite this ball until the surfaces land.

---

## What the corpus says, and where the surfaces went

`fan` carries a **prepared body** and a count; `deliver` carries `{ball,
project, handle, summary}` and `retire` `{ball, project, handle}`. All three
also take an optional `ball` upstream, which omitted means *the engine's own
focused ball* — a focus a seat does not have.

The read that names candidates already exists and already landed: **`work-diff`
rows carry `handle`, `ball_id`, `project` and `delivered`** (bl-a43a's fleet
pane). So the three acts needed no new listing, no typed handle box and no
state. A row carries a handle or it does not, and upstream's encoder is what
decides: with one it is a candidate on `attempt/<handle>`, without one it is
the ball's own claim, whose delivery obligation is the thing a fan spreads. The
controls are what the row IS.

## The fan is the start with n in the middle

Not a second start path. `Start` gains one field — the obligation, where there
is one — and every rule that value already carries applies unchanged: one
outstanding at a time, a refusal retires it with the goal back in the box, an
unread answer takes it back. Exactly two things change: the staging receipt
composes `fan` rather than `prompt`, and the fan's answer composes one ordinary
`prompt` per candidate.

The gate matters and is stated: **two `prepare` acts in flight cannot be told
apart**, because the receipt carries no correlation with the gesture that
earned it. That is already `Start`'s rule for a second start, which is why the
fan is held there rather than in a field beside it — two fields would make
*both outstanding* a representable state only the absorb order resolves.

**The frame fires the candidates and no control does.** A `fan` MATERIALIZES n
attempt worktrees, so a candidate prepared and never fired is a worktree balls
made for nothing; firing them completes the act. What that forfeits is written
down rather than hidden: upstream's terminal fires each itself *"with whatever
variation you want between them"*, and this window fires n with one goal.
Per-candidate variation is a surface and it arrives with the ball that builds
it. A spread's receipts also select nothing — *a start focuses what it started*
is a sentence about one conversation, and n receipts would focus whichever
arrived last, which is a fact about the network.

## bl-4855 is why this could land at all

`deliver` and `retire` name no workspace anywhere in their envelopes — their
subject is a ball in a project on ONE engine — so the poster would have fanned
them over every channel this box holds, accepting one candidate on every engine
the operator is a client of. That is the hazard bl-4855 settled hours earlier,
and `Model::post_candidate` is its first customer: addressed down the channel
the pane stands on. `fan` needs none of it — it carries its workspace inside
the prepared body, which `crate::envelope` already reads and already names.

**They are doors and not rows for the same reason.** Both carry nothing but
named strings, so by the table's rule they could be rows — and must not be: a
row is a word an operator TYPES, and `lernie deliver …` would be the same fan
on the argv surface, which has no channel selector. `lernie ask` stays the door
argv has.

## Nothing is armed, and the reply is why

DESIGN §4.20 is for an act whose product is that its subject is gone.
`deliver` advances a ref by the ordinary recursive delivery — git holds what it
moved, the ball is not closed, and what its close delivers is unchanged.
`retire` *"changes no delivery target, ever"*; whether the source ref goes with
the worktree is this project's own declared retention, acting on a schedule the
operator set elsewhere. So the seat PAINTS the engine's answer rather than
predicting a policy it has not read. There is no reject control because there
is no reject op: a rejection is the absence of a delivery.

Two of `delivered`'s four identities are optional and each absence is a fact —
no `source` is a source ref that was not there, no `commit` is a delivery that
landed nothing.

## What landed

`src/verbs/candidates.rs` (three doors), `src/ui/model/start/spread.rs`,
`src/ui/fleet/candidates.rs`, `Reply::{Fanned, Delivered, Retired}` with their
`read` arms, `Notice::{delivered, retired}`, `Model::post_candidate`, the
spread stepper floored at two (upstream reads 1 and 0 as *materialize nothing*)
and its own `−1`/`+1` because a bare glyph twice on one pane is one control an
operator cannot tell from another. `corpus/{delivered,fanned,retired}.json`
move from `unreadable/` to `answers/`; all three ops join the round-trip in
`emits.rs`, with the ball-less form of each recorded there by count and reason.
DESIGN §4.35 written, seven §5 map rows, and all three parity rows deleted.
