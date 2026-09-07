//! **The turn in flight**, as rows — the live fold the follow lane delivers at
//! write cadence, where [`super`] is the committed transcript at ask cadence.
//!
//! Split from [`super`] at the design-time budget on the seam the pane itself
//! has: a turn reaches this glass by two routes, and this is the one that has
//! not settled yet. What it can say is bounded by what the lane carries — the
//! two halves of the prose, and the tool window's transitions — and the one
//! rule both routes share ([`half`]) lives here because this is the route that
//! states it.

use super::super::LIVE;
use super::Row;
use crate::reply::stream::Stream;
use crate::reply::stream::window::Window;
use crate::ui::theme::Speaker;

/// The live fold's rows: the two halves of the turn, then the tool window
/// under them.
///
/// **The window is painted in the shape the committed transcript already uses**
/// (REMOTE §5.5, PROTOCOL 15) — a row as the call is dispatched and a row when
/// its capture lands — so a turn reads the same while it is happening and
/// afterwards. What the live half cannot say is what the capture CONTAINED:
/// the lane carries an exit code and `Query::Transcript` carries the text, so a
/// seat that invented a body here would be painting the reassuring answer.
pub(super) fn streaming(stream: &Stream) -> Vec<Row> {
    let mut rows = live_rows(
        stream.thinking.as_deref().unwrap_or_default(),
        stream.text.as_deref().unwrap_or_default(),
    );
    rows.extend(stream.tools.iter().flat_map(call));
    rows
}

/// One call's rows: what was dispatched, and — once its capture has landed —
/// how it ended.
///
/// **The exit code's presence is the status** and there is no third reading, so
/// a call still in flight simply has no second row. The name is
/// `<client>_<tool>` where the call was routed (REMOTE §5.1), which is what
/// makes this pane answer *what is being run on my machines* rather than only
/// *what is being run*. A call whose opening half never reached this fold is
/// named by its id, which is what the frame carried.
fn call(window: &Window) -> Vec<Row> {
    let named = window
        .tool
        .clone()
        .unwrap_or_else(|| window.tool_use.clone());
    let opened = Row::plain(
        format!("{LIVE} → {named} {}", window.tool_use),
        window.input.clone().unwrap_or_default(),
        Speaker::Model,
    );
    let landed = window.exit_code.map(|code| {
        let ok = code == 0;
        Row {
            who: format!("{} {}", window.tool_use, if ok { RETURNED } else { FAILED }),
            said: format!("exit {code}"),
            // **One call reads the same on both routes**, which is this
            // module's own rule one noun over ([`half`]): the committed
            // `ToolResult` this row becomes is a peer's weight and wears the
            // error accent when it failed, so the live half does too. A live
            // failure painted in the model's ink and a committed one painted
            // red would be one call in two colours.
            weight: Speaker::Peer,
            failed: !ok,
            fold: None,
        }
    });
    std::iter::once(opened).chain(landed).collect()
}

/// The two words a settled call is said in — the committed transcript's own,
/// so one call reads the same on both routes.
const RETURNED: &str = "returned";
const FAILED: &str = "failed";

/// **The one rule for a half of a turn, and its one home.** Reasoning and
/// answer are each a row of its own and each omitted when it is empty: a model
/// that has only thought so far, or one that answered without reasoning. An
/// empty half is simply no row — never a blank one, which would claim something
/// was said.
///
/// It is stated once because a turn reaches this pane by two routes — the live
/// fold at write cadence, the committed entry at ask cadence — and a second
/// copy of the filter is a rule one of the two will stop obeying. It did: the
/// committed path painted a `(thinking)` header over nothing for every empty
/// thinking block the engine emitted, which reads as *the model thought
/// something and this seat lost it* (bl-beb7).
pub(super) fn half(speaker: &str, mark: &str, said: &str) -> Option<Row> {
    (!said.is_empty()).then(|| {
        Row::plain(
            if mark.is_empty() {
                speaker.to_owned()
            } else {
                format!("{speaker} ({mark})")
            },
            said.to_owned(),
            Speaker::Model,
        )
    })
}

/// The two halves of a turn in flight, by [`half`]'s rule.
pub(super) fn live_rows(thinking: &str, text: &str) -> Vec<Row> {
    [("thinking", thinking), ("", text)]
        .into_iter()
        .filter_map(|(mark, said)| half(LIVE, mark, said))
        .collect()
}

#[cfg(test)]
mod tests;
