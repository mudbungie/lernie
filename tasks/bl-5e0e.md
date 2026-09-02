+++
title = "the enrollment symbol is drawn at 4 points per module, and at 1x scaling every module bleeds half a point into its neighbours"
created = 1788329656
updated = 1788329656
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Found while diagnosing bl-6952, measured rather than eyeballed, and filed
separately because it is a legibility defect in its own right whether or not it
is the black box that ball was opened for.

## The measurement

A realistic REMOTE §8.4 envelope — six fields, PEM as minted — was built and
encoded through this crate's own encoder. It is **1681 bytes**, which picks a
**153-module** symbol. `src/ui/enroll.rs` draws it at `MODULE = 4.0` points with
a 4-module quiet zone, so the symbol occupies a **644-point square** over the
central panel — the one pane that covers the middle of the window — and about
half its modules are dark.

Two things follow and both are measurements, not opinions.

- **Each module is drawn as its own filled rectangle, and egui feathers every
  fill.** Tessellated at 1 point per pixel — an ordinary non-HiDPI display —
  one 4.0-point module comes out as a **5.0-point footprint**: half a point of
  anti-aliased bleed on every side, on a 4-point pitch. So every light module
  between two dark ones loses a quarter of its width to its neighbours' feather
  from each side, and one enclosed on all four sides loses roughly 44% of its
  area. At 1.5 and 2.0 points per pixel the bleed falls to 0.33 and 0.25 points
  and the symbol sharpens, so **this is worst exactly where module pixels are
  scarcest**, which is also where a camera needs them most.
- **The whole symbol tessellates to a single 95,524-vertex mesh.** That is not
  itself a fault here — `egui_glow` 0.29.1 draws with `glow::UNSIGNED_INT`, so
  the u16 index ceiling this would otherwise cross is not in play, and that was
  checked rather than assumed. It is recorded because it is the cost of one
  draw call per module, every frame the pane is open.

## Why it matters

The symbol's whole job is to be read by a phone camera, once, before the
material is forgotten. A module that bleeds into its neighbour is contrast the
decoder does not get, and the pane offers no way to make the symbol bigger —
`MODULE` is a constant and the pane neither scales to its available width nor
offers a zoom.

## What this is not

It is not a claim that the symbol is unreadable — no decode was attempted
against a real camera, and asserting one from these numbers would be the same
guessing this investigation refused. The right shape of a fix is likely to make
the module size a function of the space the pane actually has rather than a
constant, so the symbol is drawn as large as the pane allows; the honest first
step is a decode attempt at each candidate size.