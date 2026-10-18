//! **`act:<op>` — the machine token a control carries so drift is catchable**
//! (yog's `docs/PARITY.md` §4).
//!
//! The operator's requirement is interaction parity between this seat and the
//! android client: *if something is interactable in one it must exist in the
//! other*, caught mechanically rather than noticed by hand. The subject is
//! exactly what crosses the §8.5 control boundary — an **op** — and the
//! authority for which ops owe a seat a control is yog's help table, published
//! through the `surface` field of the vendored corpus (`reply/help.json`).
//!
//! So a control that fires op `V` carries the literal token `act:V`, and the
//! meta-assertion in [`crate::snapshot::parity`] reads them back off the real
//! accessibility tree. **The op token is the one name that already exists
//! everywhere** — help row, envelope, corpus filename — so nothing here is a
//! translation table, and every call site spells the op by naming the verb's
//! own `word` rather than by writing a string.
//!
//! # Why it is `author_id` and not the label
//!
//! The visible label stays a human word: `send`, `nudge`, `start`. AccessKit's
//! `author_id` is defined for exactly this — *"a way for application authors to
//! identify this node for automated testing purpose"* — so it is machine
//! metadata that no screen reader announces and no operator sees. Putting the
//! token in the label instead would publish a wire spelling into the face.
//!
//! # One control may fire more than one op, so the token list is a list
//!
//! Two of this seat's controls compose a second gesture the walk can never see
//! as its own control. Start fires `prepare` and, when the server's reply
//! lands, the frame fires `prompt` (`crate::ui::model::start`); selecting a
//! conversation causes both the `transcript` read and the `follow` lane. The
//! tokens ride the one control that begins each, space separated, which is
//! PARITY §3's rule that the ledger's unit is the op and never the widget.
//!
//! # Why the write is `cfg(test)` and why that costs nothing
//!
//! `egui::Context::accesskit_node_builder` exists only when `egui/accesskit` is
//! on, and by the standing dev-only ruling recorded in `Cargo.toml` that
//! feature is raised in `[dev-dependencies]` alone — **the released seat links
//! no accesskit and therefore publishes no accessibility tree at all**. There
//! is no node in a release build for a token to be written to, so the write is
//! compiled where the tree exists and nowhere else.
//!
//! What is NOT conditional is the **call**, and that is the point of the seam:
//! the decision about which ops a control fires is recorded at the control, in
//! the same expression that paints it, and a control added without one is a
//! control the parity gate reddens for. The alternative — a table mapping
//! labels to ops — is the translation table PARITY §4 exists to avoid, and it
//! would drift the first time a label was reworded.

/// **The reserved namespace.** A token outside it is not this ledger's, and a
/// token inside it that names no corpus op fails the assertion — the same
/// two-direction discipline the leak fixtures hold.
pub const PREFIX: &str = "act:";

/// **The `author_id` a control firing `ops` carries**, and the one spelling of
/// the format: `act:` on each op, space separated.
///
/// Space separated rather than comma, because whitespace is what
/// `str::split_whitespace` reads back with no delimiter to get wrong, and an op
/// token never contains one (`crate::verbs::Verb::word` is a wire op).
pub fn tokens(ops: &[&str]) -> String {
    ops.iter()
        .map(|op| format!("{PREFIX}{op}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// **Tag one control with every op its activation fires.**
///
/// Call it with the response the widget handed back, in the frame that painted
/// it: the node the token lands on is the one egui created for that widget in
/// this pass, so the label, bounds and actions are already on it and this adds
/// the one field nobody else writes.
pub fn tag(response: &egui::Response, ops: &[&str]) {
    // Named here, in every build, rather than in a release-only arm: the
    // release lib is also linked by the integration tests, which never paint,
    // so a line only that build carries is a line no test can ever reach.
    let _ = (response, ops);
    #[cfg(test)]
    {
        let author = tokens(ops);
        let _ = response
            .ctx
            .accesskit_node_builder(response.id, move |node| node.set_author_id(author));
    }
    bridge(response);
}

/// **Record the seat a control is**, so an instrument can find it from the
/// tree (DESIGN §4.46 item 2). Every painter calls it through
/// `theme::paint::told`, and every tag through [`tag`], so the set the
/// instruments walk is the set of controls painted — not the tagged subset.
pub fn bridge(response: &egui::Response) {
    let _ = response;
    #[cfg(test)]
    response.ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<Bridge>(egui::Id::new(BRIDGE))
            .insert(response.id.value(), response.id);
    });
}

/// **Every seat recorded so far**, by the tree id it answers to.
#[cfg(test)]
pub(crate) fn bridged(ctx: &egui::Context) -> Bridge {
    ctx.data(|data| data.get_temp::<Bridge>(egui::Id::new(BRIDGE)))
        .unwrap_or_default()
}

/// **The way back from a tree node to the seat egui holds for it** (DESIGN
/// §4.45 item 6). AccessKit's node id IS the egui id's value, and egui will
/// say nothing about an id it is only handed as a number — so a tag records
/// the seat beside the token, and [`seat_of`] answers the tree's id with the
/// seat the lit instrument asks egui about: where it stands, and whether the
/// pointer at its centre is on it rather than on whatever covers it.
#[cfg(test)]
pub(crate) type Bridge = std::collections::BTreeMap<u64, egui::Id>;

/// Where the bridge is kept, in the context's own memory.
#[cfg(test)]
const BRIDGE: &str = "the act bridge";

/// **The seat a tree node is**, or nothing where no tag recorded one.
#[cfg(test)]
pub(crate) fn seat_of(ctx: &egui::Context, node: u64) -> Option<egui::Id> {
    ctx.data(|data| data.get_temp::<Bridge>(egui::Id::new(BRIDGE)))
        .and_then(|bridge| bridge.get(&node).copied())
}
