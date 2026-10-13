//! **The accordion**: which engine is open, when each was last opened on this
//! seat, and the wall last aimed under each (DESIGN §4.39).
//!
//! Split from [`super`] at the design-time budget on the seam that module's own
//! doc draws: [`super`] is the module list, the pieces that are a subject of
//! their own live beside it, and this is the arrangement of the one list the
//! window has.
//!
//! **It is the seat's own and crosses no boundary** (§4.39). §4.25 rules that a
//! pin crosses, because a pin is an assertion about the world; this is the
//! other case, and §4.6 draws the line: a channel is a client-side fact, named
//! by this box and held by no other, so the order this box shows its channels
//! in can be nobody else's and there is no engine to assert it into. It goes in
//! the place file ([`crate::place`]).
//!
//! **At most one engine is open, so the arrangement is a single fact — WHICH —
//! and not a set of flags to keep consistent.** Opening one closes the other by
//! construction rather than by a rule anybody has to keep.

use std::collections::BTreeMap;

use super::{Engine, Model};

/// **How the engines are arranged**, and the whole of what the place file
/// keeps about them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Engines {
    /// **Which engine the operator last opened**, where this seat still holds
    /// one by that name. `None` is a seat nobody has told, which is the first
    /// run — [`Model::engine_open`] answers that from the aim.
    pub open: Option<String>,
    /// **What each engine's last opening RANKS as** — a count that rises by
    /// one with every opening on this seat.
    ///
    /// A rank and not a clock reading, because ordering is the only question
    /// ever asked of it: two engines opened inside one millisecond would tie
    /// under a wall clock and be ordered by their names, which is the one
    /// thing the tie-break is for. A counter cannot tie, needs no clock to
    /// inject, and round-trips through the place file as the integer it is.
    pub opened: BTreeMap<String, u64>,
    /// **The wall last aimed under each engine**, by the address a gesture
    /// carries — what an opening aims at, so an engine an operator comes back
    /// to is the engine they left.
    pub aimed: BTreeMap<String, String>,
}

impl Engines {
    /// **The engines in the order the pane paints them** — the pure function
    /// over which one is open, when each was last opened, and their names.
    ///
    /// The open one first, then most recently opened first, the name breaking
    /// a tie. An engine nobody has opened here ranks below every engine
    /// somebody has, which is the honest reading of *never*.
    pub fn order(&self, names: &[String], open: Option<&str>) -> Vec<String> {
        let mut out = names.to_vec();
        out.sort_by_key(|name| self.rank(name, open));
        out
    }

    /// Where one engine stands, as a key a sort reads.
    fn rank(&self, name: &str, open: Option<&str>) -> (u8, std::cmp::Reverse<u64>, String) {
        (
            u8::from(open != Some(name)),
            std::cmp::Reverse(self.opened.get(name).copied().unwrap_or_default()),
            name.to_owned(),
        )
    }

    /// **Record an opening.** The rank is one past the highest anybody holds,
    /// so the engine just opened is the most recently opened by construction.
    pub(super) fn opening(&mut self, name: &str) {
        let next = self.opened.values().copied().max().unwrap_or_default() + 1;
        self.opened.insert(name.to_owned(), next);
        self.open = Some(name.to_owned());
    }
}

impl Model {
    /// Every engine this seat holds, by its lead's name (`super::fold`).
    fn engine_names(&self) -> Vec<String> {
        self.folded()
            .iter()
            .map(|engine| engine.name().to_owned())
            .collect()
    }

    /// **Which engine is open**, and the first run is a derivation rather than
    /// a fix-up: nobody has said, so the aim's own engine is open, and with no
    /// aim the first by name. A name this seat no longer holds is inert, which
    /// is [`crate::place`]'s whole rule about a stale place.
    pub fn engine_open(&self) -> Option<String> {
        let names = self.engine_names();
        let held = |name: Option<&String>| name.and_then(|name| self.lead_of(name));
        held(self.engines.open.as_ref())
            .or_else(|| held(self.aim.as_ref().map(|aim| &aim.channel)))
            .or_else(|| names.iter().min().cloned())
    }

    /// **The roster in the order the pane paints it** — [`Engines::order`]
    /// read back as the sections themselves, so the paint and the keyboard's
    /// cursor track cannot disagree about it.
    pub fn engine_rows(&self) -> Vec<Engine> {
        let open = self.engine_open();
        let mut rows = self.folded();
        rows.sort_by_key(|engine| self.engines.rank(engine.name(), open.as_deref()));
        rows
    }

    /// **Open one engine**, which closes whichever was open.
    ///
    /// It aims the wall last aimed under this engine, and its first by
    /// `crate::ui::roster::ordered` where this seat has never aimed one — so
    /// an open engine is never open over nothing. An engine holding no wall
    /// this seat can address aims nothing and leaves the aim where it was.
    pub fn open_engine(&mut self, name: &str) {
        self.engines.opening(name);
        self.standing = None;
        if let Some(aim) = self.opening_aim(name) {
            self.aim_at(&aim.channel, &aim.address);
        }
    }

    /// **Begin a conversation on this wall** — the whole of what `+ new`
    /// under every wall in the switcher does (§4.42 item 7, which is §4.39's
    /// `+` moved under the wall it begins on: `crate::ui::roster::wall::NEW`).
    ///
    /// It aims the wall, which clears the selection, because the composer's
    /// start mode is *a wall and no conversation* (§4.38) and this control is
    /// how an operator reaches it without hunting for a way to deselect; and
    /// asks for the caret in the box, which is `crate::ui::model::fill`'s own
    /// door one box over.
    ///
    /// **The engine's opening ranks** even though it is already open — the
    /// walls stand only under the open engine — because a start is a use, and
    /// the engine an operator just began a conversation on is the one they
    /// are using.
    ///
    /// **It crosses no wire and carries no `act:` token** (§4.16): an aim, a
    /// selection and a focus are views, and views are out of the parity
    /// contract. What the start itself fires is still the composer's own
    /// control, still tagged there.
    pub fn begin_at(&mut self, aim: &crate::ui::Aim) {
        let lead = self.engine_of(&aim.channel);
        self.engines.opening(&lead);
        self.aim_at(&aim.channel, &aim.address);
        self.fill = Some(crate::ui::Fill::Goal);
    }

    /// **Begin founding a workspace** — what the switcher's `+ new workspace`
    /// does (§4.42 item 7; bl-673e). The selection goes, because a founding
    /// is a start and a start is *no conversation*; the caret is asked for;
    /// and the box reads its next Enter as `<name> <goal>`
    /// (`crate::ui::composer::found`). Like [`Self::begin_at`] it is a view
    /// and crosses no wire.
    pub fn begin_founding(&mut self, channel: &str) {
        self.deselect();
        self.founding = Some(channel.to_owned());
        self.fill = Some(crate::ui::Fill::Goal);
    }

    /// **Found it**: the draft's first word is the new workspace's name and
    /// the rest its first conversation's goal. The start is the ordinary one
    /// aimed at a wall that does not exist yet — a `prepare` naming an
    /// unknown workspace routes to this box's own engine (REMOTE §8.2) and
    /// founds it there, registering this client (REMOTE §4) — so the receipt
    /// selects the conversation exactly as any start's does. A draft with no
    /// goal after the name fires nothing, the start's own rule.
    pub fn found(&mut self) {
        let typed = self.draft.trim().to_owned();
        let (Some(channel), Some((name, goal))) =
            (self.founding.clone(), typed.split_once(char::is_whitespace))
        else {
            return;
        };
        self.aim_at(&channel, name);
        goal.trim().clone_into(&mut self.draft);
        self.stage(name);
    }

    /// The wall an opening aims at: the one last aimed under it while this
    /// seat can still address it, else its first by `ordered` — down
    /// whichever of the engine's channels addresses it (`super::fold`).
    fn opening_aim(&self, name: &str) -> Option<super::Aim> {
        let engine = self
            .folded()
            .into_iter()
            .find(|engine| engine.name() == name)?;
        let walls: Vec<super::Aim> = crate::ui::roster::ordered(&engine.walls())
            .iter()
            .filter_map(|row| engine.aim(row))
            .collect();
        let last = self.engines.aimed.get(name);
        walls
            .iter()
            .find(|aim| Some(&aim.address) == last)
            .or_else(|| walls.first())
            .cloned()
    }
}

#[cfg(test)]
mod tests;
