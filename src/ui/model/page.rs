//! **The settings overlay's pages, and which one stands** (DESIGN §4.48
//! item 7). One overlay, a tree of pages: three sections, eleven pages, and
//! every page that used to be an overlay of its own is a page of this one.
//!
//! **The page is derived, not kept.** Four of the pages are the panes that
//! hold state of their own — the tuning's rows and draft, the login's
//! following, the config's file and draft, the clients listing — and each of
//! those stands exactly while its field does, which is also what its standing
//! read is keyed on (`crate::state::traffic::Open`). The other five hold
//! nothing of their own and ride in [`Lookup::Settings`]. So [`Model::page`]
//! is a query over state the model already held, and a second field saying
//! which page is open would be two representations of one fact.
//!
//! **Every word stands.** A page with a word is opened by that word
//! ([`Page::word`] → `crate::ui::slash::fire`), so the aim gate, the uncover
//! and the read happen as they would from the box; the tree's row is one more
//! door to the same word, and the menu, the reach bound and PARITY's one-door
//! rule are unchanged.

use super::{Listing, Lookup, Model};

/// **The three sections of the tree** — `Group` in code, because the
/// queue's `Section` is already a name here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// This seat's own knobs.
    Seat,
    /// The aimed workspace's facts, each a read and acts on the wire.
    Workspace,
    /// The servers this box holds.
    Servers,
}

impl Group {
    /// The three, in the tree's order.
    pub const ALL: [Self; 3] = [Self::Seat, Self::Workspace, Self::Servers];

    /// The word on the tree's header and in the breadcrumb.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Seat => "Seat",
            Self::Workspace => "Workspace",
            Self::Servers => "Servers",
        }
    }

    /// Its pages, in the tree's order.
    pub(crate) fn pages(self) -> impl Iterator<Item = Page> {
        Page::ALL
            .into_iter()
            .filter(move |page| page.group() == self)
    }
}

/// **The eleven pages.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// The knob table's four pages (`crate::settings::Page`).
    Seat(crate::settings::Page),
    /// `/roles` (`crate::ui::tuning`).
    Roles,
    /// `/login` (`crate::ui::login`).
    Providers,
    /// `/config` (`crate::ui::config`).
    Config,
    /// `/clients` (`crate::ui::clients`).
    Clients,
    /// **Register this box** (`crate::ui::register`): the URL field, the
    /// button and the outcome. No word: it opens directly, aimed or not.
    Register,
    /// `/offers` (`crate::ui::offers`): minting an offer for a new device,
    /// and approving or declining the pending ones.
    Offers,
    /// `/doctor` (`crate::ui::doctor`).
    Doctor,
}

impl Default for Page {
    fn default() -> Self {
        Self::Seat(crate::settings::Page::default())
    }
}

impl Page {
    /// The eleven, in the tree's order.
    pub const ALL: [Self; 11] = [
        Self::Seat(crate::settings::Page::Look),
        Self::Seat(crate::settings::Page::Input),
        Self::Seat(crate::settings::Page::Bar),
        Self::Seat(crate::settings::Page::Mode),
        Self::Roles,
        Self::Providers,
        Self::Config,
        Self::Clients,
        Self::Register,
        Self::Offers,
        Self::Doctor,
    ];

    /// Which section it is under.
    pub const fn group(self) -> Group {
        match self {
            Self::Seat(_) => Group::Seat,
            Self::Roles | Self::Providers | Self::Config | Self::Clients => Group::Workspace,
            Self::Register | Self::Offers | Self::Doctor => Group::Servers,
        }
    }

    /// The word on the tree's row and at the breadcrumb's end.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Seat(page) => page.label(),
            Self::Roles => "roles",
            Self::Providers => "providers",
            Self::Config => "config",
            Self::Clients => "clients",
            Self::Register => "register",
            Self::Offers => "offers",
            Self::Doctor => "doctor",
        }
    }

    /// What the page is about: the tree row's help, and what the search reads.
    pub(crate) const fn help(self) -> &'static str {
        match self {
            Self::Seat(page) => page.help(),
            Self::Roles => "what each of this workspace's roles runs on, and how hard it thinks",
            Self::Providers => "what this workspace can sign in to, and the sign-in itself",
            Self::Config => "the files this workspace's policy is written in",
            Self::Clients => "which machines may execute for this workspace",
            Self::Register => {
                "joins this box to a server with a one-time registration URL and a key made here"
            }
            Self::Offers => "lets a new device join: mint an offer, then approve it by its word",
            Self::Doctor => "whether this box is wired, one check at a time",
        }
    }

    /// **The word that opens it**, for the pages that are a word's — the
    /// four with state of their own and the doctor, whose word posts the
    /// read. The rest open directly.
    pub(crate) const fn word(self) -> Option<&'static str> {
        match self {
            Self::Roles => Some(crate::ui::tuning::WORD),
            Self::Providers => Some(crate::ui::login::WORD),
            Self::Config => Some(crate::ui::config::WORD),
            Self::Clients => Some(crate::ui::clients::WORD),
            Self::Offers => Some(crate::ui::offers::WORD),
            Self::Doctor => Some(crate::ui::doctor::WORD),
            Self::Seat(_) | Self::Register => None,
        }
    }
}

/// **The overlay's search** (§4.48 item 7): what is typed in its box, and
/// the row a picked hit jumps to — the knob's key, taken by that row's painter
/// on the frame it scrolls to it (`crate::ui::settings::rows`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seeking {
    pub text: String,
    pub jump: Option<&'static str>,
}

impl Model {
    /// **Which page the settings overlay shows**, or none where it is down.
    /// A ladder over fields no two of which float together
    /// ([`Model::covered`]), so the order is never spent.
    pub fn page(&self) -> Option<Page> {
        if self.tuning.is_some() {
            Some(Page::Roles)
        } else if self.login.is_some() {
            Some(Page::Providers)
        } else if self.configuring().is_some() && !self.docked() {
            // **Docked, the config pane is the context rail's** (§4.48 item
            // 8), not a page of this overlay.
            Some(Page::Config)
        } else if self.showing(Listing::Clients) {
            Some(Page::Clients)
        } else if self.showing(Listing::Offers) {
            Some(Page::Offers)
        } else if let Some(Lookup::Settings(page)) = self.lookup {
            Some(page)
        } else {
            None
        }
    }

    /// **Whether `page` can be opened now**: a page with a word is offered
    /// exactly while its word is on the menu (an aim, for the workspace's
    /// four), and the rest always.
    pub fn offers(&self, page: Page) -> bool {
        page.word().is_none_or(|word| {
            crate::ui::slash::rows(self)
                .iter()
                .any(|row| row.word == word)
        })
    }

    /// **Turn the overlay to `page`**: by its word where it has one, else
    /// directly — over whatever stood, since one overlay stands at a time.
    /// Answers whether it opened.
    pub fn turn_to(&mut self, page: Page) -> bool {
        if let Some(word) = page.word() {
            return crate::ui::slash::fire(self, word, "");
        }
        self.uncover();
        self.lookup = Some(Lookup::Settings(page));
        true
    }

    /// **The mark's door** (bl-036a): the overlay where it was last, or
    /// *Seat › look* where that page is not offered now — the one a seat
    /// aimed at nothing can always open.
    pub fn reopen_settings(&mut self) {
        if !self.turn_to(self.visited) {
            self.turn_to(Page::default());
        }
    }
}

#[cfg(test)]
mod tests;
