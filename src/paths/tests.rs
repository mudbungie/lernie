//! Which of two variables names the root, and what it means when neither does.

use super::{data_root, keep_of, root_of, state_root};
use std::ffi::OsString;
use std::path::PathBuf;

fn stated(text: &str) -> OsString {
    OsString::from(text)
}

/// The XDG variable names the root outright, and wins over the convention's
/// default: a box that has already placed application data somewhere has
/// already answered this question.
#[test]
fn the_xdg_variable_names_the_root_outright() {
    assert_eq!(
        root_of(Some(stated("/var/lib/app")), Some(stated("/home/u"))),
        Ok(PathBuf::from("/var/lib/app/lernie"))
    );
}

/// Without it, the convention's default under the home directory.
#[test]
fn without_it_the_root_is_the_convention_under_the_home_directory() {
    assert_eq!(
        root_of(None, Some(stated("/home/u"))),
        Ok(PathBuf::from("/home/u/.local/share/lernie"))
    );
}

/// **An empty variable is an unset one.** A launcher that exports a name with
/// no value has said nothing, and treating it as a root would name the
/// filesystem root's own directory.
#[test]
fn a_variable_with_no_value_has_said_nothing() {
    assert_eq!(
        root_of(Some(stated("")), Some(stated("/home/u"))),
        Ok(PathBuf::from("/home/u/.local/share/lernie"))
    );
    assert!(root_of(Some(stated("")), Some(stated(""))).is_err());
}

/// Neither set is a refusal naming both variables — never a relative guess,
/// which would put an operator's certificates wherever the launcher happened
/// to start the process.
#[test]
fn neither_variable_set_refuses_and_names_them_both() {
    let refusal = root_of(None, None).expect_err("refused");
    assert!(refusal.contains("XDG_DATA_HOME"), "{refusal}");
    assert!(refusal.contains("HOME"), "{refusal}");
    assert!(refusal.contains("will not guess"), "{refusal}");
}

/// The process-edge read is the pure rule applied to this process's own
/// environment, and nothing else — it joins paths and touches no disk.
#[test]
fn the_edge_read_is_the_rule_applied_to_this_process() {
    assert_eq!(
        data_root(),
        root_of(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
    );
}

/// **The fence has a third surface and this is it.** A box that ran the
/// pre-fence engine may hold a `$XDG_DATA_HOME/lernie` from that era, and this
/// crate takes the same directory rather than conceding the name it was given.
/// What makes that benign is that the seat reads exactly two paths under its
/// root and the engine's home held neither shape — so the assertion worth
/// holding is that the root is the plain crate name and the reading is narrow.
#[test]
fn the_data_root_is_the_crate_s_own_name_under_the_convention() {
    let root = root_of(Some(stated("/data")), None).expect("named");
    assert_eq!(root, PathBuf::from("/data/lernie"));
    assert_eq!(
        crate::channel::entries::flat(&root),
        PathBuf::from("/data/lernie/wire"),
        "everything this crate reads is under one subdirectory of the root"
    );
}

/// **Two roots, one ladder, and the split is the hazard the module doc names.**
/// What the operator carried here cannot be rebuilt by anything on this box;
/// what the window remembers about itself can be deleted at any time for the
/// cost of a selection. A regenerable subtree beside irreplaceable material
/// makes the second look like the first, so they never share a root.
#[test]
fn what_the_seat_generates_never_lands_beside_what_the_operator_carried() {
    let data = root_of(None, Some(stated("/home/u"))).expect("named");
    let kept = keep_of(None, Some(stated("/home/u"))).expect("named");
    assert_eq!(data, PathBuf::from("/home/u/.local/share/lernie"));
    assert_eq!(kept, PathBuf::from("/home/u/.local/state/lernie"));
    assert!(!kept.starts_with(&data), "{kept:?} under {data:?}");
    assert!(!data.starts_with(&kept), "{data:?} under {kept:?}");
}

/// The state root takes its own variable outright, on the same ladder — and an
/// empty one has said nothing there too.
#[test]
fn the_state_variable_names_its_root_and_an_empty_one_says_nothing() {
    assert_eq!(
        keep_of(Some(stated("/var/state")), Some(stated("/home/u"))),
        Ok(PathBuf::from("/var/state/lernie"))
    );
    assert_eq!(
        keep_of(Some(stated("")), Some(stated("/home/u"))),
        Ok(PathBuf::from("/home/u/.local/state/lernie"))
    );
}

/// **A different sentence, because a different thing is lost.** Nothing an
/// operator carried here is at stake in this root, so the refusal says what it
/// costs rather than warning about certificates landing somewhere nobody chose.
#[test]
fn an_unnameable_state_root_costs_only_what_the_window_remembers() {
    let refusal = keep_of(None, None).expect_err("refused");
    assert!(refusal.contains("XDG_STATE_HOME"), "{refusal}");
    assert!(refusal.contains("HOME"), "{refusal}");
    assert!(refusal.contains("remembers between runs"), "{refusal}");
    assert!(
        !refusal.contains("certificates"),
        "nothing irreplaceable is at stake here: {refusal}"
    );
}

/// The process-edge read of the state root is the same rule applied to this
/// process's own environment, and nothing else.
#[test]
fn the_state_edge_read_is_the_rule_applied_to_this_process() {
    assert_eq!(
        state_root(),
        keep_of(std::env::var_os("XDG_STATE_HOME"), std::env::var_os("HOME"))
    );
}
