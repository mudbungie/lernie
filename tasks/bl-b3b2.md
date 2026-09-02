+++
title = "a region of the window paints nothing and the desktop flickers through it"
created = 1788329772
updated = 1788329904
claimant = "Vellum"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Continues bl-6952. **Found, reproduced headlessly, fixed, and guarded in both
directions.**

## The two operator facts that named the class

- The `enroll a box` pane is open and sits to the RIGHT of the black box, so
  the black region is the middle pane — the conversation list — and not the
  enrollment. The QR is eliminated.
- **The region flickers transparently**: the desktop shows through on some
  frames. Paint that chose a wrong colour could never show the desktop. Only
  paint that never happened can. So this is a COVERAGE defect, not a colour
  defect.

## The cause

A side panel paints a frame sized to its own `max_width`, and then reserves
`inner_response.response.rect.max` from the layout — the inner `Ui`'s own
extent, which grows with its content. Those are two different numbers the
moment the pane's content is wider than the cap `ui::shell::widths` hands it,
and the central panel begins after the **reserved** edge rather than the
**painted** one.

**The strip between them is covered by no panel at all.** Nothing paints it, so
what shows there is the window surface's own clear — black where it is opaque,
the compositor behind it on the frames the alpha path lets through. That is the
flicker, exactly.

The content that overflowed is the **row headline**. `Ui::selectable_label`
lays its text with `TextWrapMode::Extend`, hard-coded, and a horizontal layout
would force Extend anyway — so a conversation whose name is long makes the
pane's inner `min_rect` wider than the pane is allowed to be.

Measured, before the fix, with the enrollment open:

    window   painted grounds                       hole
      1400   x0..280   x280..600   x688..1400      88 pt   (long name)
       800   x0..177   x177..380   x407..800       27 pt
       700   x0..140   x140..289   x370..700       81 pt
       600   x0..140   x140..280   x370..600       90 pt

A full-height vertical strip in the middle of the window, widening as the
window narrows. The width of the widest headline changes as answers land on the
750 ms beat, so the strip's width changes with it and closes when every row is
short enough — which is the flicker's period.

**Why it was never seen before 0.1.4:** 0.1.2 spoke protocol 4 and 0.1.3 spoke
5 against an engine speaking 6, and the handshake is fail-closed. 0.1.4 is the
first build whose conversation list has ever held real rows with real names.
With no rows the pane's content is one short label and nothing overflows. The
defect is first-*seen*, not newly-*introduced* — bl-6952 established that from
the diff before the pane was known.

## The fix

The headline truncates. It is laid by `egui::Label::truncate().sense(click)`
instead of `selectable_label`, and the selection ink is painted under the run —
the widget that drew it before is the one that cannot truncate, and a selected
row is unchanged to look at. The pane now takes exactly the width it paints.

## The guard, both directions

`the_panel_grounds_tile_the_window_with_no_gap_for_the_clear_to_show_through`
asserts the invariant rather than the widget: at six window widths, the
full-height panel grounds tile the window edge to edge. It **fails on the
unfixed tree** and names the hole —

    at 1400 points a strip of x600.0..688.0 is painted by no panel

— and passes at every width with the fix. That is the acceptance criterion for
this defect and it could not have been written before bl-6952 gave
`crate::paint_probe` a fills projection: the harness walked painted glyphs
only, and a hole has no glyphs.