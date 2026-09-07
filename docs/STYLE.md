# lernie — Style

What the desktop seat LOOKS like, and where that is decided (bl-73d2; operator
verdict 2026-09-06 on the window as it stood: *borderline unusable, so ugly
and unintuitive*). The language's home is the phone seat's
`docs/STYLE.md` in yog-android — its three rules, its palette, its six-state
ruling and its anatomy are adopted here verbatim and are not restated where
they hold unchanged. This file states **the desktop's information
architecture** and **the desktop's deltas**, each with its reason. It is
normative: a pane that departs from it is a defect, and the fix is here or in
`src/ui/theme.rs`, never on the pane.

`docs/DESIGN.md` says what each pane IS and why it exists; this file says what
it looks like and where a person's eye goes first.

## 1. The three rules, and what they cost this window

1. **Colour means state** — six states, six accents, no seventh (operator
   ruling 2026-09-06): green asks for you, blue is working, purple is the
   model generating, orange is a note that wants salience, grey is done,
   red is an error that will not mend itself. Hierarchy comes from spacing,
   size and alignment. So the selection highlight is a brand tint, not a
   colour of its own; a heading is a size, not a hue; and the one row a
   person needs is the one wearing green.
2. **No outlines.** No pane is boxed and no control is bordered. The three
   columns meet at a `HAIRLINE`; a row is bare ground and tints under the
   pointer; a control is a `SURFACE` tint with no stroke; the one stroke on
   the glass is the brand ring on **whatever holds the keyboard** — the field
   with the caret, the control a Tab landed on, the row it landed on.
   **Delta:** the phone rings only the caret's field, because a thumb never
   tabs; a desktop is driven from the keyboard, and a focus that cannot be
   seen is a focus nobody can use (DESIGN §4.11). The window
   it replaces was a grid of grey chips — every act a bordered button, every
   pane a framed box — and a grid of chips is a face with no hierarchy.
3. **One module.** `src/ui/theme.rs` holds every byte, size and gap;
   `src/ui/theme/visuals.rs` is the one adapter into egui, installing the
   tokens as a `Style` at the top of every frame; `rules/no-literal-colour.yml`
   refuses a colour spelled anywhere else. A pane asks for a colour by the
   name of a state and for a gap by the name of a step.

## 2. Information architecture: what a person sees first

**The window is three columns, left to right: where, which, what.** The
roster (every workspace this seat can reach), the aimed workspace's
conversations, and the selected conversation with its composer under it.
That shape stands: it is the desktop's one structural delta from the phone
(which shows one column at a time), it is what a wide display is for, and
the width policy that yields the two lists to the conversation's floor and
collapses to one column below it (DESIGN §4.11, `src/ui/shell/policy.rs`)
already answers every width. What was wrong was never the columns; it was
that nothing in them was weighted.

**The eye lands on green.** A workspace row and a conversation row that are
asking for the operator wear the attention accent — the count in green ink
and a three-point green rule at the row's left edge — and nothing else on the
glass is green. A window with five conversations waiting is five green rules
in a column, readable from across the room; a window with none is a window
with nothing green on it, which is the same fact read the other way.

**Navigation reads from layout, not labels.** A row is a place you go: click
it or arrow to it and the column to its right fills in. The three headings
say which column the arrows belong to by wearing the brand mark, and a
column's rows say what state each thing is in by their rule. No control
says *open* or *select*; position says it. The window-level acts — the queue
of what is waiting, the trail, the board, the engines' vocabulary, a search,
and asking every channel again — are a band of entries at the top of the
roster column, where a desktop's global navigation is expected, styled as
rows and not as chips. The queue's entry wears the attention accent while
anything is waiting, so the one control most worth pressing is the one that
is coloured.

**A conversation's facts are arranged, not listed.** Over the transcript, the
subject line: its name in body ink, its state word in that state's accent,
then — one step weaker — which model answered and what it has spent. Under
the transcript, the composer: the field, and beside it `send` in the brand.
Under the field, the controls band: the acts that advance the turn first
(`send`, `interrupt`, `nudge`), then the acts on the conversation as an
object (`records…`, `stop`, `revoke`, `restore`, `retarget`, `flag`,
`delete`), the two parameter boxes beside the act each fills, the
destructive one last (DESIGN §4.20). Every control keeps its `act:` token
(DESIGN §4.16): what changed is the paint, never the ledger.

**The composer glows when it is your turn.** While the selected conversation
is asking for the operator the field is tinted with the attention accent
and its hint says so; otherwise it rests on `SURFACE`. The field is where
the answer goes, so it is where the asking is shown.

## 3. Ground, elevation, ink, accents — adopted verbatim

The palette is the phone's, byte for byte (`src/ui/theme.rs`): `GROUND`,
`SURFACE`, `RAISED`, `HAIRLINE`; `INK`, `INK_WEAK`, `INK_FAINT`; the six
accents under `State`, with `Rest` wearing `INK_WEAK` on purpose and the
brand being the working blue. The four properties the phone asserts are
asserted here too (`theme::tests`): the ladder climbs, ink reads on every
rung at 7:1 / 4.5:1, no accent is louder than two-thirds saturated, and an
unknown wire word is unstyled rather than restyled.

**The wire's states read onto the six** (`theme::state_of`, `theme::tone_ink`,
REMOTE §11): a held driver is `Inference`, streaming is `Working`, stopped is
`Error`, settled is `Rest`; the row tones `good`/`bad`/`live`/`in-flight`
read the same way the phone reads them, and `plain`/`weak` are the ink
scale. **A speaker is told by weight** (`theme::speaker`): the operator's rule
is the brand, the model's is full ink, a peer's is weak, an ended sender's is
faint.

The QR symbol keeps black on white (`QR_PAPER`, `QR_INK`) whatever the
window is, because a camera reads it and no theme may tune it.

## 4. Scales: the desktop's deltas

Spacing is the phone's, `XS 4 · S 8 · M 12 · L 16 · XL 24`, plus one step
under it, `XXS 2` (`theme::space`). **Delta:** the default item gap is `S`
across and `XXS` down, and a control is padded `S` across and `XXS` down.
Three columns of single-line rows at the phone's gap is a window one third
empty; a desktop reads a stack of short lines as prose rather than as a list
of targets; and a covering pane must fit the narrowest shape the layout
promises (DESIGN §4.32's density constraint), which the phone's smallest
step overruns.

Type is four sizes and no fifth (`theme::type_scale`): `SMALL 11 · MONO 12 ·
BODY 13 · HEADING 18`. **Delta:** every size sits two points under the
phone's, because a desktop is read at arm's length and holds three columns
of prose where the phone holds one.

**Delta:** a list row stands `ROW 24` points tall where the phone's touch
target is 48 — a pointer's target is not a thumb's, and a list of
conversations at 48 a row scrolls twice as much; a control is its word plus
its padding, no taller than it needs. A block's corner is `RADIUS 6` in
proportion; a state's rule is `RULE 3`, as on the phone.

## 5. Anatomy

**A column heading**: `HEADING` size, `INK_WEAK` at rest, `INK` with the
brand `›` mark when the arrows are its. No fill, no rule under it.

**A row** (`theme::paint::row`): full width, `ROW` tall, its words at the
left edge plus `M`, elided at the width it has. Bare ground at rest,
`SURFACE` under the pointer, `RAISED` tint plus a brand rule when it is the
aimed or selected one; a state rule at its left edge when it has a state to
show. Its words carry more than one ink on one line — name in `INK`, count
in `INK_WEAK`, the asking count in the attention accent — and that order is
the hierarchy. Never a `Button` and never a `selectable_label`: both draw a
box.

**A threaded row** (the conversation list): the row above at its depth, with
the connector rails in `INK_FAINT` — the phone's L-shaped threading idiom —
its first line in the row's tone and every line after it in `INK_WEAK`.

**A block** (the transcript): aligned, ruled, never bubbled. Every block
starts at the same left edge; a `RULE`-wide line in the speaker's weight
stands beside its header; the body wraps at the width it has; a tool result
folds to its head in `INK_WEAK` with the unfold control carrying the size of
what is hidden.

**The composer**: a `SURFACE` field, no stroke at rest, the brand ring when
focused, the attention tint when the conversation is asking; `send` beside
it in the brand. **A control**: `SURFACE` fill, no stroke, `RADIUS` corners,
its word in `INK`; `RAISED` pressed or open; `INK_FAINT` disabled and still
on the glass; the attention accent on an armed irreversible act.

**A notice**: one sentence under the top edge in the state's ink — `Error`
for a refusal or an unreadable answer, `Annotation` for a note — with no
fill and no box, and a dismiss at its end.

**A covering pane** (records, queue, trail, enrollment and the rest): the
pane's heading, its subject line one step weaker, its close as a control at
the right of that line, and sections separated by `L` of space and a
`HAIRLINE` — never by a framed group.

## 6. Where it is asserted

`src/ui/theme.rs` and its adapter are under the 100% floor: the ladder, the
contrast floors, the softness bound, the six-and-no-seventh, the wire
readings and the type scale are `theme::tests`. Every pane assertion reads
the glyphs that reached the glass (`src/paint_probe.rs`), and the ink a run
was painted in is on the same read (`paint_probe::Seen::ink`), so *the
asking row is green* is a test rather than a picture. `make snapshots`
writes the kittest matrix under `target/snapshots/` for eyes; the pictures
that adopted this language are in the usability campaign's round-3 notes.
