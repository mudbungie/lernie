+++
title = "CI is red on main: coverage 5416/5417, src/ui/enroll/symbol.rs line 99 uncovered since b723c2f"
created = 1788583308
updated = 1788583344
claimant = "Animations-V"
priority = 1
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
## The failure

`CI / linux → make ci` fails at the coverage step:

    || Uncovered Lines:
    || src/ui/enroll/symbol.rs: 99
    || src/ui/enroll/symbol.rs: 23/24
    99.98% coverage, 5416/5417 lines covered
    Error: "Coverage is below the failure threshold 99.98% < 100.00%"

Red since b723c2f (the last green tree was 59e8fb0, which reported
`src/ui/enroll/symbol.rs: 24/24` and `100.00% coverage, 5365/5365`), and still
red at the tip 30ca81e. The release job is skipped behind it, so 0.1.29 is the
last published version and every later landing is stuck.

## The line

    fn module(rect: egui::Rect, x: usize, y: usize, pitch: f32) -> egui::Rect {
        egui::Rect::from_min_size(
            rect.min + egui::vec2(span(x + QUIET, pitch), span(y + QUIET, pitch)),
            egui::vec2(pitch, pitch),          // <- line 99
        )
    }

`module` is called directly by `a_module_sits_inside_its_own_quiet_zone` and
again by every rasterizing test through `dark`. There is no path on which the
function runs and that argument does not.

## Root cause

It is bl-f83f's phantom, in a second file. Line 99 holds **only a
sub-expression of the call around it** — no statement of its own — so llvm-cov
has no statement to attribute the count to and the region is scored from
whatever the codegen left there. The denominator is identical on both sides
(5417 lines), so nothing about the tree's coverable surface changed: the same
line scores 1 locally and 0 on the runner. Measured, on the tree at 30ca81e:
`make coverage` locally reports `100.00% coverage, 5417/5417` while the runner
reports `99.98%, 5416/5417`.

b723c2f did not touch this file. It added nine tests, which changed codegen
enough for the attribution to flip — the same way bl-f83f's two lines flipped
between machines rather than between trees. That is what makes it a phantom
rather than a regression: nothing about the tree got less tested.

## The fix

The same remedy bl-f83f applied to `qr::matrix`: name the values, so no line
holds only an argument.

    let corner = rect.min + egui::vec2(span(x + QUIET, pitch), span(y + QUIET, pitch));
    let size = egui::vec2(pitch, pitch);
    egui::Rect::from_min_size(corner, size)

A `let` is a statement, so there is nothing left to mis-attribute.

## What this does not solve

The shape is produced by the formatter — rustfmt breaks a call that overruns
the width, and any argument that lands alone on a line is a candidate. So there
is no rule to add that forbids it without fighting `fmt`, and no way to prove
the tree holds no other one; the class is found by a runner disagreeing with a
local box, which is what the 100% floor is for. What can be said is that this
is the second sighting and both were fixed the same way, so the shape is now
named in the code beside the fix.