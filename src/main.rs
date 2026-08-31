//! The process entry, and nothing else.
//!
//! It reads argv, hands it to [`lernie::cli::run`], folds in this process's own
//! environment where the decision needs it, writes the verdict's text to the
//! stream the verdict names, and exits with its code. There is no decision
//! here — every one of them is in the library, where a test reads it back as a
//! value. That is what earns this file its place as the single exclusion in
//! `tarpaulin.toml`.
//!
//! The one thing that is the entry point's rather than the library's is
//! **this process's own environment**, folded once into the data root. Every
//! function below this line takes the root as an argument, which is why the
//! suite never needs a process to test one.

use std::process::ExitCode;

use lernie::cli::{Decided, Stream, Verdict};
use lernie::state::Link;
use lernie::ui::{Chunk, Model};

fn main() -> ExitCode {
    let verdict = match lernie::cli::run(std::env::args().skip(1).collect()) {
        Decided::Say(verdict) => verdict,
        Decided::Entries => rooted(lernie::seat::listing),
        Decided::Ask(envelope) => rooted(|root| lernie::seat::ask(root, &envelope)),
        Decided::Start { address, goal } => {
            rooted(|root| lernie::seat::start(root, &address, &goal))
        }
        Decided::Enroll {
            workspace,
            name,
            grade,
        } => rooted(|root| lernie::seat::enroll(root, &workspace, &name, &grade)),
        Decided::Window => rooted(window),
    };
    match verdict.stream {
        Stream::Out => println!("{}", verdict.text),
        Stream::Err => eprintln!("{}", verdict.text),
    }
    ExitCode::from(verdict.code)
}

/// Fold this process's environment into a data root and hand it over, or answer
/// the refusal that says the root is not named.
fn rooted(act: impl FnOnce(&std::path::Path) -> Verdict) -> Verdict {
    match lernie::paths::data_root() {
        Ok(root) => act(&root),
        Err(reason) => Verdict::failed(reason),
    }
}

/// **Open the window.**
///
/// The seat's face, on the channels this box holds. It seeds the roster off the
/// disk — which dials nothing, so a box with no engine up still paints what it
/// has — and then runs the frame loop over `lernie::ui::render`.
///
/// The three off-frame threads are started around it and stopped when it
/// closes; the frame's own side of them is one `settle` at the top of every
/// update, which files what landed, hands over what a click composed, and
/// publishes what to ask next.
///
/// This is the entry point's own work — a native event loop is process state,
/// exactly as argv and the environment are — which is why it lives in the one
/// file `tarpaulin.toml` excludes. It decides nothing: every value it paints
/// comes from the library, where a test reads it back.
fn window(root: &std::path::Path) -> Verdict {
    // **The seat's own place, off the STATE root** — never the data root, which
    // holds only what the operator carried here (`lernie::paths`). A root this
    // process cannot name costs a forgotten selection and nothing else, so it
    // is folded to nothing rather than refused.
    let keep = lernie::paths::state_root().ok();
    let mut model = Model {
        roster: lernie::seat::channels(root)
            .into_iter()
            .map(Chunk::of)
            .collect(),
        aim: keep.as_deref().and_then(lernie::place::read),
        ..Model::default()
    };
    let link = Link::new(BEAT);
    // The standing set is published by a settle, so the first pass would ask
    // nothing at all until a frame had run. One settle here is what makes the
    // window's first paint the answer to a question rather than a blank.
    link.settle(&mut model);
    let workers = lernie::offframe::run(&link, root);
    let ran = eframe::run_native(
        lernie::mark::APP_ID,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                // **The app id is the whole of what a Wayland compositor has to
                // go on** (`lernie::mark`): it matches this against the
                // installed desktop entry and reads the mark off that. The icon
                // below reaches X11 and nothing else, so both are set and
                // neither is redundant.
                .with_app_id(lernie::mark::APP_ID)
                .with_icon(egui::IconData {
                    rgba: lernie::mark::rgba(ICON_PX),
                    width: u32::from(ICON_PX),
                    height: u32::from(ICON_PX),
                }),
            ..eframe::NativeOptions::default()
        },
        Box::new({
            let link = link.clone();
            move |_| {
                Ok(Box::new(Seat {
                    model: std::mem::take(&mut model),
                    link,
                }))
            }
        }),
    );
    link.stop();
    for worker in workers {
        let _ = worker.join();
    }
    // **The last frame's aim is already across the lock**: the standing set is
    // published on every settle, so where the operator was pointed is a
    // projection of what the workers were reading and needs no plumbing of its
    // own — and writing it here, once, is what keeps it off the frame.
    if let Some(said) = keep
        .as_deref()
        .and_then(|at| lernie::place::write(at, link.standing().aim).err())
    {
        eprintln!("the seat could not keep its place: {said}");
    }
    match ran {
        Ok(()) => Verdict::ok(String::new()),
        Err(e) => Verdict::failed(format!("the window would not open: {e}")),
    }
}

/// How big the embedded mark is rasterized. One size, because the toolkit
/// takes one and scales it: a set of sized rasters is what a hicolor theme is
/// for, and that is the installed SVG's job rather than this one's.
const ICON_PX: u16 = 256;

/// How often the seat asks. Human cadence: a roster an operator glances at does
/// not want to be a second faster, and the two surfaces that move quicker than
/// a glance are held reads rather than a tighter loop.
const BEAT: std::time::Duration = std::time::Duration::from_millis(750);

/// The frame loop's whole body: settle what the threads brought, render, and
/// ask to be woken in a beat.
struct Seat {
    model: Model,
    link: Link,
}

impl eframe::App for Seat {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.link.settle(&mut self.model);
        lernie::ui::render(ctx, &mut self.model);
        // A seat paints what somebody else is doing, so the frame has to come
        // round without an operator touching anything — and no faster than the
        // asker can bring something new.
        ctx.request_repaint_after(self.link.beat());
    }
}
