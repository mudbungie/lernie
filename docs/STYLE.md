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

**The window is one column beside a bar** (DESIGN §4.42, bl-7859; §4.44,
bl-f772; it was two columns until the operator's ruling of 2026-10-12, and
three before that). The **bar** on the left lists the aimed workspace's
conversations: a glyph column a few characters wide until the pointer is
over it, then its rows in full — state glyph, name, the first line beneath in
weak ink, the age at the right — collapsed, the glyph and as much of the
name as the dragged width holds — staying open 0.75 s after the pointer
leaves, or always while pinned (the pin at its head, persisted); its edge
drags either width, and both are remembered (DESIGN §4.45). `+ new` is
at its head on the right; its foot is the **workspace**, the one place a
workspace is chosen, a dropup opening upward. Over the transcript stands a
**tab strip**, one tab per conversation opened in that workspace, the
selected one over a brand hairline, a faint `×` closing each. The baseline is the
Hermes Agent terminal UI, aped part for part, with one order reversed
(DESIGN §4.45, bl-4792): the transcript fills the window, the **prompt
line** stands directly under it, and the one-line **status rule** stands at
the bottom, under the prompt, so the words typed are beside the words they
answer. Nothing else is on the glass until it is asked for, and it is asked
for by a slash verb or a key — and comes as a framed box floating over the
transcript, above the prompt. A width buys wrapping and nothing else; there is
no shape and no yield; the one dragged edge is the bar's.

**The eye lands on green.** The ask above the prompt is a green-framed box;
a dropup's check is green for the moment its list is fresh; the prompt line
glows green while the selected conversation is asking; the switcher's asking
rows wear `?` in green. A window with nothing waiting has nothing green on it.

**The rule is the conversation's own line** (DESIGN §4.42, item 11; §4.44
took the workspace off it). One line: `— provider ⏶ | model ⏶ | effort ⏶ |
priority ⏶ | state · elapsed | tokens in · out | cost | context N% — name`.
The four selectors
come first, in body ink, each with `⏶` in faint ink inside its door, where a
hand reaches to open it (the body font has no `▾`); a click opens a **dropup** — a small framed list over the
segment, opening upward — showing what this seat holds at once and asking
for it again, while the segment's mark is `↻` in the annotation accent
(the palette has no yellow) and then `✔` in the attention accent once the
answer has landed. After them, the conversation's diagnostics in weak ink,
the state first in its state's ink — where the notice stands while there is
one, and Esc puts it down; a notice longer than the line stands whole,
wrapped, just above the rule. The name at the right end is the switcher's
door. A selector or the name that holds the keyboard wears the brand ring,
and Enter or Space opens it. Nothing seat-wide stands on the rule: how many conversations wait and
how many engines answer are the seat panel's, the switcher's and
`/attention`'s. What runs out of width drops from the right — diagnostics,
then priority, then effort; provider and model never drop and are cut with
`…` inside their share instead. What you talk to is changed where
it is read.

**The prompt is a line, not a form.** `›` in the brand (the body font has no `❯`), the draft after it,
a line per line typed, the hint in weak ink while it is empty: *say it — Enter
sends · /help for commands · Ctrl+X for a conversation · Ctrl+O for the model ·
/login for a provider · Shift+Enter breaks a line*, its trailing parts dropped
one by one where the line has no room — one line, never wrapped, never a part
cut in half; *what this conversation is for — Enter begins it* and the same
ways in, in start mode; *nothing is aimed · / for a verb · Ctrl+X for a
conversation or a new workspace* with no wall aimed; *no provider yet —
/login or /config, then pick one on the rule* on a wall whose worker's
provider holds no credential (bl-7a77), where prose goes nowhere and the verbs that
need no aim still fire; *it is asking — 1–6 answers, or say it — Enter sends* while it asks, the caret
staying in the box (bl-1252). There is no send control and no row of controls under
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
seat reaches, one row each with its dial state and counts — an engine that
will not dial says why in red in place of its counts, and while it is the
aimed engine the rule's state slot says the same in red — the queue's door
`N waiting` in green while anything asks, the version, and
the ways in: *the verbs, a conversation, the model, a provider*. Under the
transcript, the prompt; under the prompt, the rule.

**A transcript is read by its gutter.** `❯` is the operator, `┊` the model,
a tree of stems the working — `+ worked · 2 calls · 1 note` at rest, open
`├─ ✓ Bash("…")` over one elided line of its output, `└─ ✗` in the error
accent, a step in flight whole — and `───` parts the turns. No block wears a side rule and
every block stands on a faint bubble that says its role (§5), and a
message this seat sent stands at the end in `INK_WEAK` until the engine's
row replaces it.

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
the hover wash (`theme::hover`, the brand at three quarters of the selection
tint) under the pointer, `RAISED` tint plus a brand rule when it is the
aimed or selected one; a state rule at its left edge when it has a state to
show. A row with lines beneath its head (`theme::paint::Row`) is one block:
the tint, the rule and the ring span every line of it. Its words carry more than one ink on one line — name in `INK`, count
in `INK_WEAK`, the asking count in the attention accent — and that order is
the hierarchy. Never a `Button` and never a `selectable_label`: both draw a
box.

**A threaded row** (the conversation list): the row above at its depth, with
the connector rails in `INK_FAINT` — the phone's L-shaped threading idiom —
its first line in the row's tone and every line after it in `INK_WEAK`.

**The status rule** (`status`): one line of `SMALL` runs parted by `|` in
`INK_FAINT`, led by `—`; the five selectors in `INK` (a placeholder such as
`model…` in `INK_WEAK`) each with its mark inside the same door — `⏶` in `INK_FAINT` at
rest, `↻` in the annotation accent while its list reloads, `✔` in the
attention accent once it has — then the state word in its state's ink and
every other diagnostic in `INK_WEAK`, the name at the right end in `INK`.
Diagnostics drop from the right first, then priority, then effort; the first
three selectors are cut instead. **The selectors and the name are links**
(`theme::paint::link`): the same run at the same width, bare at rest,
the hover wash under the pointer with the pointing hand — never a box, never a
button, because a fact you can open is still a fact.

**The dock** (`dock`): one `SMALL` line in the working accent, a link
(`theme::paint::link`) — bare at rest, the hover wash under the pointer —
that unfolds it into a row apiece in the row idiom. Every seat that senses
a click is one of these three — a row, a link or a tab, or egui's own
button — and lights (DESIGN §4.45 item 6).

**A dropup** (`status::dropup`): the overlay's frame (`SURFACE`, the hairline,
`RADIUS`) over a span of the window rather than across it, its left edge at
the segment's, its foot on the rule; the keys in `INK_WEAK` at `SMALL` on the
first line, then one row per choice in the row idiom, the row in force led by
`•`, the cursor's row selected.

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
attention accent, standing between the transcript and the prompt line
while the selected conversation asks — the rule is under the prompt
(DESIGN §4.45): the queue row's words in the queue's own inks,
then its choices as numbered rows, `1.` to at most `6.`, the chosen one
`RAISED`.

**A working** (`chat::working`): stems `├─` `└─` `│` in `INK_FAINT`; its
one line `+ worked · 2 calls · 1 note` in `INK_WEAK`, a door; a finished
call compact as a row, `✓ name("preview")` with the mark in `INK_WEAK` or
the error accent and one line of its output elided beneath; a finished note
`· who · first line…`; a step whole `● name("preview") ✓` with the dot in
the brand, a result folded to its head and unfolded in place.

**A conversation header** (`chat::subject`, retired by §4.42; the rule carries it): the name at `HEADING` in
`INK`, the state word beside it in its accent, the model-and-spend line in
`INK_WEAK`, the subtree path in `INK_WEAK`, a provider's refusal in the
error accent above the spend, and `S` of air before the transcript.

**A block** (the transcript, `chat`): aligned, guttered, and bubbled by
role (DESIGN §4.45 item 7): a fill at `RADIUS` behind the whole block, the
operator's on the brand's tint, a trail on `GROUND` inside a `HAIRLINE`,
intermediate prose on `SURFACE`, the turn's final prose on `RAISED` — the
model's three rising toward what is read, the operator's the one hued fill,
all faint (`theme::bubble`). Every block starts at the same left edge with its glyph in the gutter —
the prompt's `›` in the brand for the operator, `¦` in `INK_WEAK` for the
model, `·` in `INK_WEAK` for a peer, a tool's answer or the machinery —
then who spoke in their weight (`theme::speaker`, the error accent on a
failure) and the body as prose, laid at `LEADING` and wrapped at the width
it has — save a fenced code block, which is laid in the monospace face on
`SURFACE` at `RADIUS`, fences unpainted, never wrapped and scrolled sideways
inside the column instead (`chat::code`, bl-beef). A turn's working (its calls, results, reasoning and the prose between) is
one block on the trail's bubble that folds in three stages (DESIGN §4.45
item 5): a step in flight whole, a finished step compact to a line or two
elided at the width, a settled turn's working one counted line; a toggle
remembered. Open, each step stands each step on a painted stem
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
with + new below*, *nothing is waiting on you — a conversation that asks
will appear here* — never only what is missing. Weak because it is the absence of
content said once; a wait (*waiting to hear…*) is the same shape, because it
resolves itself and the reader needs to know only that.

**A notice**: one sentence in the state's ink — `Error` for a refusal or an
unreadable answer, `Annotation` for a note — with no fill and no box, in the
rule's state slot, or whole and wrapped just above the rule where the slot
cannot hold it beside every fact (§2); Esc dismisses it.

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
