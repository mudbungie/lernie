//! What the followed run paints: both streams, the settled exit, the
//! run-by-hand command, and the empty fold that is a reading rather than a
//! silence.

use super::{BY_HAND, COPY, FINISH, NO_RUN, OPEN, settled};
use crate::paint_probe::frame::Window;
use crate::reply::login::{Line, Signin};
use crate::test_support::window::{click, pane, signing};
use crate::ui::Model;

/// **Both streams reach the glass**, because bz writes the authorize URL and
/// every remedy to stderr — a pane that kept only stdout paints a blank.
#[test]
fn the_run_paints_both_streams_and_its_settled_exit_and_fallback() {
    let mut model = signing();
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    for word in [
        "open https://provider.invalid/auth",
        "waiting for the browser",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    let mut model = Model {
        signin: Some(Signin {
            lines: vec![Line {
                text: "no device endpoint".to_owned(),
                err: true,
            }],
            outcome: Some(78),
            fallback: Some("bz --login --provider housevendor".to_owned()),
            authorize: None,
        }),
        ..signing()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    for word in [
        &settled(78),
        BY_HAND,
        "bz --login --provider housevendor",
        "no device endpoint",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    assert_eq!(settled(0), "signed in");
}

/// **A pair with no run is a reading, not a silence** — the lane opens on one
/// empty frame and the pane says what that frame says.
#[test]
fn a_row_nobody_has_signed_in_to_says_so() {
    let mut model = Model {
        signin: Some(Signin::default()),
        ..signing()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(painted.lines().any(|line| line == NO_RUN), "{painted}");
    let mut unheard = Model {
        signin: None,
        ..signing()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut unheard);
    });
    assert!(
        !painted.contains(NO_RUN),
        "nothing has been heard at all:\n{painted}"
    );
}

/// The URL the fixture's waiting run stands on.
const URL: &str = "https://provider.invalid/auth";

/// The whole pane, as the window paints it.
fn body(model: &mut Model) -> impl FnMut(&egui::Context) + '_ {
    move |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::render(ui, model);
        });
    }
}

/// **Only a run that waits offers the three controls** (REMOTE §8.3, edition
/// 31): no `authorize`, no box — which is also the whole of the edition gate —
/// and a settled run offers nothing to finish.
#[test]
fn only_a_run_waiting_for_a_code_offers_the_page_and_the_box() {
    let offered = |model: &mut Model| {
        let painted = pane(|ui| {
            super::super::render(ui, model);
        });
        [OPEN, COPY, FINISH].map(|word| painted.contains(word))
    };
    assert_eq!(offered(&mut signing()), [true; 3]);
    let mut device = signing();
    device.signin.as_mut().expect("the run").authorize = None;
    assert_eq!(offered(&mut device), [false; 3], "a device flow");
    let mut settled = signing();
    settled.signin.as_mut().expect("the run").outcome = Some(0);
    assert_eq!(offered(&mut settled), [false; 3], "a settled run");
}

/// **The page opens in this desktop's browser, and its address can be
/// carried to another**: one control hands the URL to the toolkit's opener,
/// the other puts it on the clipboard. Neither sends anything.
#[test]
fn the_waiting_runs_url_opens_here_or_is_copied_for_elsewhere() {
    let mut model = signing();
    let window = Window::new();
    let (mut opened, mut copied) = (None, String::new());
    let mut frame = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::render(ui, &mut model);
        });
        let (open, copy) = ctx.output(|o| (o.open_url.clone(), o.copied_text.clone()));
        opened = open.map(|open| open.url).or(opened.take());
        if !copy.is_empty() {
            copied = copy;
        }
    };
    click(&window, OPEN, &mut frame);
    click(&window, COPY, &mut frame);
    assert_eq!(opened.as_deref(), Some(URL));
    assert_eq!(copied, URL);
    assert!(model.outbox.is_empty());
}

/// **The submit sends what was pasted and keeps the run**, and with nothing
/// pasted it is not live.
#[test]
fn finishing_sends_the_paste_as_the_code_and_an_empty_box_is_not_live() {
    let mut model = signing();
    let window = Window::new();
    click(&window, FINISH, body(&mut model));
    assert!(model.outbox.is_empty(), "nothing was pasted");
    *model.code_draft().expect("the box") = "pasted-code#state".to_owned();
    click(&window, FINISH, body(&mut model));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(serde_json::json!({
            "op": "login", "workspace": "home",
            "provider": "housevendor", "code": "pasted-code#state",
        }))]
    );
    assert!(model.signin.is_some(), "the same run is still followed");
}
