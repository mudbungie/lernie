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
2. **No outlines, except on what floats** (amended by DESIGN §4.42). No
   control is bordered and no pane is boxed; an overlay, the completion
   menu, the ask and the panel at the head of a transcript wear a one-point
   frame in `HAIRLINE`, because a stroke on a thing that floats says what is
   over what. The sentence that follows is the rule as it stood. The three
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

**The window is one column** (DESIGN §4.42, bl-7859; it was two until the
operator's ruling of 2026-10-12, and three before that). The baseline is the
Hermes Agent terminal UI, aped part for part: the transcript fills the window,
a one-line **status rule** stands under it, and the **prompt line** stands at
the bottom. Nothing else is on the glass until it is asked for, and it is asked
for by a slash verb or a key — and comes as a framed box floating over the
transcript, above the prompt. A width buys wrapping and nothing else; there is
no shape, no yield and no dragged edge.

**The eye lands on green.** The ask above the prompt is a green-framed box;
the waiting count on the rule is green while it is non-zero; the prompt line
glows green while the selected conversation is asking; the switcher's asking
rows wear `?` in green. A window with nothing waiting has nothing green on it.

**The rule says where you are and what it costs.** One line:
`─ state · elapsed │ model │ spend │ N waiting │ N engines ─ name`. The state
word is the wire's own, in its state's ink; the notice stands in that slot in
its ink while there is one, and Esc puts it down. The right end is the
conversation's name, the wall's where none is selected, `engines` where
nothing is aimed. There is no heading over the transcript: one pane needs no
name, and the rule carries the one fact a heading carried. **The model and the
name are doors** (DESIGN §4.42, item 10): the model opens the role tuning —
on a wall nothing has answered on it says the worker role's model where the
roles are in hand and `model…` where they are not (bl-0966) — and the name opens
the switcher. What you talk to is changed where it is read.

**The prompt is a line, not a form.** `›` in the brand (the body font has no `❯`), the draft after it,
a line per line typed, the hint in weak ink while it is empty: *say it — Enter
sends · /help for commands · Ctrl+X for a conversation · /roles for the model ·
/login for a provider · Shift+Enter breaks a line*, its trailing parts dropped
one by one where the line has no room — one line, never wrapped, never a part
cut in half; *what this conversation is for — Enter begins it* and the same
ways in, in start mode; *nothing is aimed · / for a verb · Ctrl+X for a
conversation* with no wall aimed, where prose goes nowhere and the verbs that
need no aim still fire; *it is waiting on you* while it asks. There is no send control and no row of controls under
it: the field is the control, and the verbs are typed.

**Navigation is typed.** `/` opens the completion menu — verb and summary,
narrowed as you type, Tab takes the row — and the word is the pane's noun or
the verb's word, exactly as the control it replaced read: `/records`,
`/trail`, `/board`, `/fleet`, `/clients`, `/roles`, `/config`, `/prices`,
`/login`, `/enroll`, `/doctor`, `/find`, `/attention`, `/help`, `/unmake`,
`/refresh`, `/pin`, `/stop`, `/nudge`, `/flag`, `/delete`, and `/model`
(Ctrl+O) for the role tuning by the word you reach for. Ctrl+X opens
the switcher, the one list of engines, walls and conversations, with `+ new`
under every wall. A control that opens a pane does not exist; the word does.

**A conversation's facts are a card, then a rule.** At the head of its
transcript, the conversation panel: a framed card with its name, engine and
wall, model, spend and where it hangs, one line each, the long ones behind
`▸` folds. With nothing selected the seat panel stands there: the engines this
seat reaches, one row each with its dial state and counts, the version, and
the ways in: *the verbs, a conversation, the model, a provider*. Under the
transcript, the rule; under the rule, the prompt.

**A transcript is read by its gutter.** `❯` is the operator, `┊` the model,
a tree of stems the trail — `▾ tool calls (3)`, `├─ ● Bash("…") ✓`, `└─ ✗` in
the error accent — and `───` parts the turns. No block wears a side rule and
no block is bubbled.

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
Columns of single-line rows at the phone's gap is a window one third
empty; a desktop reads a stack of short lines as prose rather than as a list
of targets; and a covering pane must fit the narrowest shape the layout
promises (DESIGN §4.32's density constraint), which the phone's smallest
step overruns.

Type is four sizes and no fifth (`theme::type_scale`): `SMALL 11 · MONO 12 ·
BODY 14 · HEADING 18`. **Delta:** the body sits one point under the phone's
and the heading two, because a desktop is read at arm's length and holds
its columns of prose side by side where the phone holds one; 13 was the first pass and
read as spartan at 1440×900 (bl-f251), and 14 is the point the density
constraint still admits. **Prose has a leading**, `LEADING 21` — one and a
half bodies — laid only under a transcript body (`theme::paint::prose`); a
row, a control and a label keep the font's own height, so that a paragraph
and a list are told apart by the air between their lines.

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

**The status rule** (`status`): one line of `SMALL` runs parted by `│` in
`INK_FAINT`, led by `─`; the state word in its state's ink, every other
segment in `INK_WEAK`, the waiting count in the attention accent while it is
non-zero, the name at the right end in `INK`. Segments drop from the right
as the width runs out, the name last. **The model and the name are links**
(`theme::paint::link`): the same run at the same width, bare at rest,
`SURFACE` under the pointer with the pointing hand — never a box, never a
button, because a fact you can open is still a fact.

**The prompt line** (`composer`): `›` in the brand at the left edge plus `M`,
the draft in `INK` beside it at `BODY`, continuation lines indented to the
draft's edge, the hint in `INK_WEAK`; `SURFACE` under it, the brand ring
while it holds the caret, the attention tint while the conversation asks.
It is as tall as its lines and never taller.

**An overlay** (`overlay`): a `SURFACE` box with a `HAIRLINE` frame and
`RADIUS` corners over the transcript, `S` under the top of the pane and `L`
in from each side, padded `S`; its name on the first line in `INK` at `BODY`
with what it is open on beside it in `INK_WEAK` and *Esc closes* at the
line's other end in `INK_WEAK` at `SMALL`; its body under that. The
completion menu is the same box as tall as the rows it lays — a window of
them around the chosen one, never a scroll — with the keys said in one
`SMALL` line above them; a row is the row above, its line `/word  summary`,
and the chosen one wears `RAISED`.

**The ask** (`ask`): the overlay's box with its frame and its name in the
attention accent, standing between the transcript and the rule while the
selected conversation asks: the queue row's words in the queue's own inks,
then its choices as numbered rows, `1.` to `6.`, the chosen one `RAISED`.

**A trail** (`chat`): stems `├─` `└─` `│` in `INK_FAINT`; the fold head
`▾ tool calls (3)` in `INK_WEAK`; a call `● name("preview") ✓` with the dot in
the brand, the mark in `INK_WEAK` or the error accent; a result folded to its
head, unfolded in place at `MONO`.

**A conversation header** (`chat::subject`, retired by §4.42; the rule carries it): the name at `HEADING` in
`INK`, the state word beside it in its accent, the model-and-spend line in
`INK_WEAK`, the subtree path in `INK_WEAK`, a provider's refusal in the
error accent above the spend, and `S` of air before the transcript.

**A block** (the transcript, `chat`): aligned, guttered, never bubbled.
Every block starts at the same left edge with its glyph in the gutter —
the prompt's `›` in the brand for the operator, `¦` in `INK_WEAK` for the
model, `·` in `INK_WEAK` for a peer, a tool's answer or the machinery —
then who spoke in their weight (`theme::speaker`, the error accent on a
failure) and the body as prose, laid at `LEADING` and wrapped at the width
it has. A trail stands under its fold head, each step on a painted stem
(the roster's own rails and elbows) with `●` in the brand for a call, `✓`
in `INK_WEAK` or `✗` in the error accent for a result, folded to its head
with the unfold control carrying the size of what is hidden. A `response`
line on an elbow parts a trail from the prose after it; a `HAIRLINE` parts
one turn from the next; `S` of air parts everything else.

**The panel at the head** (`chat::panel`): a `SURFACE` card with a
`HAIRLINE` frame and `RADIUS` corners, `M` across and `S` down inside, one
fact a line — the name in `INK`, the rest in `INK_WEAK` — and the seat's
card the same with each engine's line in the attention accent while any of
its walls asks.

**The composer** (`theme::paint::composer`): a `SURFACE` field
`COMPOSER_ROWS` lines tall, no stroke at rest, the brand ring when focused,
the attention tint when the conversation is asking; `send` inside it at the
bottom right, in the brand on the brand's tint. Its height is allocated from
the rows and grows only from the words, never from the panel it stands in.
**A control**: `SURFACE` fill, no stroke, `RADIUS` corners, its word in
`INK`; `RAISED` pressed or open; `INK_FAINT` disabled and still on the
glass; the attention accent on an armed irreversible act. **A compact
control** is a glyph and a word in one run — `theme::worded` — and the glyph
is a token of the theme, asserted to have a glyph in the body font.

**A strip** (the window's acts, the aimed wall's acts): compact controls in
one wrapped row, a short verb each — `refresh`, `verbs…`, `roles…`,
`unmake…` — never a sentence, so that at the roster's own width the six
stand on two lines and the eight on three. A verb that opens a pane carries
the ellipsis; the one that only asks does not.

**An empty state** (`theme::paint::empty`): one sentence in `INK_WEAK`,
wrapped, that names the next act — *no conversations here yet — begin one
with + on the engine*, *nothing is waiting on you — a conversation that asks
will appear here* — never only what is missing. Weak because it is the absence of
content said once; a wait (*waiting to hear…*) is the same shape, because it
resolves itself and the reader needs to know only that.

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
