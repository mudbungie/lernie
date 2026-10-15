+++
title = "the transcript's bubbles by role, and a trail that folds itself once its calls are done"
created = 1792034031
updated = 1792034556
claimant = "Bubbling"
parent = "bl-4792"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-b7cb"
on = "claim"
+++
DESIGN §4.45 items 5 and 7, amending §4.42 item 4; STYLE §5 *a block* and *a trail* already state the result.

**The bubble is a token and a projection, never a painter's count.** `theme::bubble` (`theme/bubble.rs`) names the four fills once, derived from the palette: the operator's turn the brand at half a tint (fainter than the hover and the selection, so a wash over it still reads), a trail on `GROUND` inside a `HAIRLINE` stroke — the lowest step needs the ring to be visible on the ground it stands on — intermediate prose on `SURFACE`, final prose on `RAISED`. Which bubble a row gets is a field on `Row` set in the projection: a row is born on the bubble its kind names (`rows/role.rs::born` — operator prose, a trail step, else the final rung) and one pass over each turn's shape (`role::settle`) demotes prose a further step follows, or every prose of the running turn, to intermediate. A turn is an operator row and everything to the next one; the running turn is the last one, because the engine's `Streaming` entry rides the open response and so is always at the end. The same pass marks every row of the running turn `live`, which is the one fact the trail's fold reads for its default.

**The fold's default is the turn's state; the toggle is the toolkit's memory.** `chat/trail.rs` (split from `chat.rs` at the cap) opens a trail by default while any of its steps is live and shuts it once the turn settled; an explicit click is remembered in `ui.data` keyed by the trail's index, as before, and overrides the default either way. The head is `theme::paint::tab` — a link at the body size — so it wears the hover wash; the fold buttons are egui's own and wear egui's hover.

**The block is one shape in the anatomy.** `theme::paint::bubbled` is an `egui::Frame` at `RADIUS` with `S` inside, spanning the column, so the gutter glyph stands inside the fill and the roles read as one ramp down the pane rather than four widths of speech balloon. The hairline between turns and the `response` line are unchanged.

**What this did to the detector, and why.** The bubble's inner margin moved the transcript's rows eight points down, and in the `slashing` and `unpinning` worlds at the phone and narrow sizes the operator's header landed in the flat margin of the completion menu floating over it — `snapshot::blank` complained the glass was blank where the layout put a word. It was blank by design: the box is laid after the transcript and stands over its foot. The tree cannot tell a box from the pane under it (egui lays both on one layer and registers no container nodes), and *a fill painted after the run* alone is exactly the slab the detector exists for. So `paint_probe::covered` asks the paint order for a fill painted later that holds the run AND has a run of its own laid inside it after it — a box carries words, a slab carries nothing — and the detector skips what a box covers while a slab stays caught (`snapshot::tests` keeps proving the slab). One projection on the one walk, per `rules/no-hand-rolled-paint-walk.yml`.

**Residual, filed here rather than solved.** A fenced code block (`chat::code`, `SURFACE` at `RADIUS`) inside intermediate prose stands on `SURFACE` over `SURFACE` and is told from its bubble only by its monospace face; inside final prose it reads as a dip. The palette has no fifth rung and §4.45 asks for no new literal, so it stands as it is until the eye says otherwise.

Tests: the four fills read off the probe under the words of a transcript with an operator turn, intermediate prose, a trail and final prose, with the glyph inside the fill and the trail ringed; a streaming turn's prose intermediate and the same words final once settled; a settled trail shut and opened by its head, a running trail open and a toggle remembered on later frames; the head lit under the pointer; the projection's pass as pure rows; the theme's ramp climbing and the operator's fill the brand's hue at a lower alpha than the hover.