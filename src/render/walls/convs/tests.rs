//! The fold: what a root stands for, and what the listing still says about it.

use super::conversations;
use crate::reply::convs::{AgentState, ConvRow, Tone};

/// One row, as thin as the listing needs.
fn row(display: &str, depth: u64, members: u64) -> ConvRow {
    ConvRow {
        root_id: format!("c-{display}"),
        display: display.to_owned(),
        name: Some(display.to_owned()),
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 9,
        attention: 0,
        members,
        depth,
        tone: Tone::Plain,
        failure: None,
    }
}

/// **Compactors fold under the conversation they compact** (bl-96cd, bl-00f5).
///
/// One session answered twelve rows, seven of them machinery the operator
/// never started. The listing is what a person scans to find out what is
/// happening, so what it prints is the conversations; what hangs under one is
/// named on its root's own line and costs no row of its own.
#[test]
fn a_subtree_is_named_under_its_root_rather_than_listed_beside_it() {
    let said = conversations(&[
        row("rsroman", 0, 4),
        row("TrellisCopper", 1, 1),
        row("ShoalTeacup", 1, 1),
        row("FrostyHillside", 1, 1),
        row("ballfix", 0, 1),
    ]);
    let rows: Vec<&str> = said
        .lines()
        .filter(|line| line.contains("quiescent"))
        .collect();
    assert_eq!(rows.len(), 2, "two roots, not five rows: {said}");
    assert!(
        said.contains("3 under it: TrellisCopper, ShoalTeacup, FrostyHillside"),
        "{said}"
    );
    // The root still says how big the subtree is, off the field the engine
    // already carries.
    assert!(said.contains("4 members"), "{said}");
}

/// **A root with nothing under it says nothing about a subtree** — the empty
/// arm is an absence, exactly as every other clause on this surface is.
#[test]
fn a_root_standing_alone_gains_no_line() {
    let said = conversations(&[row("ballfix", 0, 1)]);
    assert!(!said.contains("under it"), "{said}");
}

/// **A deep row with no root above it is its own row**, never dropped. It can
/// only mean the answer began mid-forest, and a listing that quietly lost rows
/// would be worse than one that indents oddly.
#[test]
fn a_descendant_with_no_root_above_it_is_still_printed() {
    let said = conversations(&[row("brave-fox", 2, 3), row("ballfix", 0, 1)]);
    assert!(said.contains("brave-fox"), "{said}");
    assert!(said.contains("2 deep"), "{said}");
    assert!(!said.contains("under it"), "{said}");
}

/// **A fork spreading many candidates costs one line, elided the way a preview
/// is** — and the elision is marked, which is what says to ask `--json`.
#[test]
fn a_wide_subtree_is_elided_on_one_line() {
    let mut rows = vec![row("spread", 0, 13)];
    for n in 0..12 {
        rows.push(row(&format!("candidate-number-{n}"), 1, 1));
    }
    let said = conversations(&rows);
    assert!(said.contains("12 under it:"), "{said}");
    assert!(said.contains('…'), "the elision is marked: {said}");
    assert_eq!(
        said.lines()
            .filter(|line| line.contains("under it"))
            .count(),
        1,
        "{said}"
    );
}
