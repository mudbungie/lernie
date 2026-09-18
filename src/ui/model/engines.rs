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

use super::{Chunk, Model};

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
    fn opening(&mut self, name: &str) {
        let next = self.opened.values().copied().max().unwrap_or_default() + 1;
        self.opened.insert(name.to_owned(), next);
        self.open = Some(name.to_owned());
    }
}

impl Model {
    /// Every engine this seat holds, in the order it was asked.
    fn engine_names(&self) -> Vec<String> {
        self.roster
            .iter()
            .map(|chunk| chunk.channel.name.clone())
            .collect()
    }

    /// **Which engine is open**, and the first run is a derivation rather than
    /// a fix-up: nobody has said, so the aim's own engine is open, and with no
    /// aim the first by name. A name this seat no longer holds is inert, which
    /// is [`crate::place`]'s whole rule about a stale place.
    pub fn engine_open(&self) -> Option<String> {
        let names = self.engine_names();
        let held = |name: Option<String>| name.filter(|name| names.contains(name));
        held(self.engines.open.clone())
            .or_else(|| held(self.aim.as_ref().map(|aim| aim.channel.clone())))
            .or_else(|| names.iter().min().cloned())
    }

    /// **The roster in the order the pane paints it** — [`Engines::order`]
    /// read back as the sections themselves, so the paint and the keyboard's
    /// cursor track cannot disagree about it.
    pub fn engine_rows(&self) -> Vec<Chunk> {
        let open = self.engine_open();
        let mut rows = self.roster.clone();
        rows.sort_by_key(|chunk| self.engines.rank(&chunk.channel.name, open.as_deref()));
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
        if let Some(address) = self.opening_aim(name) {
            self.aim_at(name, &address);
        }
    }

    /// **Begin a conversation on this engine** — the whole of what the `+` at
    /// the right edge of its row does (§4.39,
    /// `crate::ui::roster::engine::BEGIN`).
    ///
    /// It opens the engine, which aims the wall this seat last aimed under it;
    /// clears the selection, because the composer's start mode is *a wall and
    /// no conversation* (§4.38) and this control is how an operator reaches it
    /// without hunting for a way to deselect; and asks for the caret in the
    /// box, which is `crate::ui::model::fill`'s own door one box over.
    ///
    /// **Opening it is not a side effect**: a start is a use, and the engine
    /// an operator just began a conversation on is the one they are using, so
    /// it takes the rank every other opening takes.
    ///
    /// **It crosses no wire and carries no `act:` token** (§4.16): an aim, a
    /// selection and a focus are views, and views are out of the parity
    /// contract. What the start itself fires is still the composer's own
    /// control, still tagged there.
    pub fn begin_on(&mut self, name: &str) {
        self.open_engine(name);
        // Explicitly, and not as `aim_at`'s doing: an engine holding no wall
        // this seat can address aims nothing, and the start mode is still
        // where the operator asked to be.
        self.conversation = None;
        self.column = crate::ui::Column::Conversation;
        self.fill = Some(crate::ui::Fill::Goal);
    }

    /// The wall an opening aims at: the one last aimed under it while this
    /// seat can still address it, else its first by `ordered`.
    fn opening_aim(&self, name: &str) -> Option<String> {
        let chunk = self
            .roster
            .iter()
            .find(|chunk| chunk.channel.name == name)?;
        let walls: Vec<String> = crate::ui::roster::ordered(&chunk.walls)
            .iter()
            .filter_map(|row| chunk.channel.address(row))
            .collect();
        self.engines
            .aimed
            .get(name)
            .filter(|last| walls.iter().any(|wall| wall == *last))
            .cloned()
            .or_else(|| walls.first().cloned())
    }
}

#[cfg(test)]
mod tests;
