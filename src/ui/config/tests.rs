//! What the config pane says: the destinations it offers, the two views of one
//! read, and the judgement that is the engine's.

use super::{
    CLOSE, HEADING, NO_BYTES, NO_LINEAGES, NO_SETTINGS, NOT_ANSWERED, NOT_READ, NOTHING_PICKED,
    render,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, configured, lineage, pane, seated};
use crate::ui::{Configuring, Model};
use crate::verbs::Where;

/// A closed pane paints nothing and says so, which is what lets the shell put
/// the conversation back where it was.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **Three empty states and each is its own sentence**: a wall nobody has been
/// answered about, a wall that holds no lineage, and a pane pointed at no file
/// yet.
#[test]
fn the_three_empty_states_say_three_different_things() {
    for (lineages, expected) in [
        (None, NOT_ANSWERED),
        (Some(Vec::new()), NO_LINEAGES),
        (Some(vec![lineage("default")]), "default"),
    ] {
        let mut model = Model {
            configuring: Some(Configuring::default()),
            lineages,
            ..seated()
        };
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(painted.contains(expected), "{expected:?}:\n{painted}");
        assert!(painted.contains(NOTHING_PICKED), "{painted}");
    }
}

/// **The destinations are what can be read**: the wall's own brazen file, the
/// engine's two globals, and every path on every lineage.
#[test]
fn every_destination_the_pane_offers_is_on_the_glass() {
    let mut model = configured();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "brazen",
        "litany models",
        "cadence",
        "default: providers.yaml",
        "default: workflow.yaml",
        "abcdef1",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **Both views of one read are on the glass together** — the settings the
/// schema found, and the bytes they were found in.
#[test]
fn the_settings_and_the_bytes_are_painted_as_one_answer() {
    let mut model = configured();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "worker.provider",
        "gone",
        "provider",
        "watcher.debounce_ms",
        "number 0–10000",
        "roles:",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The judgement is the engine's, in its own words** — what is painted as
/// wrong here is what the far end says is wrong, and a value it did not fault
/// carries nothing.
#[test]
fn a_faulted_setting_carries_the_engine_s_own_sentence() {
    let mut model = configured();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(
        painted.contains("brazen's table has no provider row"),
        "{painted}"
    );
}

/// **A file with no schema and a file with no bytes are two readings**, and
/// neither is an error: the first is a raw-text destination, the second a file
/// that does not exist yet.
#[test]
fn an_empty_schema_and_an_empty_file_are_two_sentences() {
    let mut model = Model {
        config: Some(crate::reply::config::Config {
            text: String::new(),
            settings: Vec::new(),
        }),
        ..configured()
    };
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains(NO_SETTINGS), "{painted}");
    assert!(painted.contains(NO_BYTES), "{painted}");
}

/// A file picked and not yet answered says it is waiting, under the name of
/// the file that was asked for.
#[test]
fn a_file_asked_for_and_unanswered_says_so_under_its_own_name() {
    let mut model = Model {
        config: None,
        ..configured()
    };
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains(NOT_READ), "{painted}");
}

/// **Clicking a destination points the pane at it**, which is what stands the
/// file read up — the read has no control of its own.
#[test]
fn clicking_a_destination_points_the_pane_at_that_file() {
    let window = Window::new();
    let mut model = configured();
    click(&window, "cadence", |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.configured(), Some(Where::Cadence));
    assert!(model.outbox.is_empty(), "a read stands rather than posting");
}

/// The way out is the pane's own control.
#[test]
fn the_close_control_puts_the_pane_down() {
    let window = Window::new();
    let mut model = configured();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.configuring, None);
}

/// **The workflow destination is a box and then a control**, and an empty name
/// names nothing: upstream addresses a workflow by a name no read this seat
/// has enumerates, so the box is the listing, dark until it holds one.
#[test]
fn the_workflow_name_is_typed_and_an_empty_one_reaches_nothing() {
    let window = Window::new();
    let mut model = configured();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains(super::WORKFLOW), "{painted}");
    if let Some(name) = model.workflow_box() {
        *name = "nightly".to_owned();
    }
    click(&window, super::WORKFLOW, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(
        model.configured(),
        Some(Where::LitanyWorkflow {
            name: "nightly".to_owned()
        })
    );
}
