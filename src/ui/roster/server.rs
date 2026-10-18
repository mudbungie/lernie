//! **One server's row**, the control that begins a conversation on it, and the
//! cursor track the whole list makes (DESIGN §4.39).
//!
//! Split from [`super`] at the design-time budget on the seam the pane now
//! has: [`super`] is the list and what a section says, `workspace` is one
//! workspace, and this is the server itself — the row an operator clicks to
//! open one, and the order the rows stand in.
//!
//! **The cursor here is not the selection, and it is the one place in this
//! window where that is true** (§4.39). Landing on a server's row only stands
//! there: opening is an act, and a walk that opened every server it moved
//! through would be a walk nobody could use to reach the one below. So the row
//! carries the brand rule while the cursor is on it, and
//! [`crate::ui::keys`]'s Enter calls [`Model::open_server`] — the same door
//! this file's click calls, which is what keeps it from being a second
//! surface.

use crate::ui::model::Server;
use crate::ui::{Aim, Model, Stand, theme};

/// **One stop on the window's cursor track**: a row the walk only stands on
/// ([`Stand`] — a server's own row, `+ new`, `+ new workspace`), a workspace
/// under the open one, or a conversation under the aimed workspace.
///
/// Landing on a workspace aims it and landing on a conversation selects it, exactly
/// as a click does; landing on a [`Stand`] only stands there (§4.39) — the walk
/// must not open every server it passes through, nor begin on every workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// A row the cursor stands on, and Enter fires.
    Stand(Stand),
    /// A workspace under the open server, by what a gesture must carry.
    Workspace(Aim),
    /// A conversation under the aimed workspace, by its root id.
    Conversation(String),
}

impl Step {
    /// **Whether the window's cursor is on this stop right now.**
    ///
    /// The three are mutually exclusive by construction rather than by a rule
    /// anybody keeps: standing on a server clears the aim's claim to the
    /// cursor, and aiming clears the selection (`crate::ui::model::acts`), so
    /// at most one row answers yes.
    pub fn holding(&self, model: &Model) -> bool {
        match self {
            Self::Stand(stand) => model.standing.as_ref() == Some(stand),
            Self::Workspace(aim) => {
                model.standing.is_none()
                    && model.conversation.is_none()
                    && model.aim.as_ref() == Some(aim)
            }
            Self::Conversation(root_id) => {
                model.standing.is_none() && model.conversation.as_deref() == Some(root_id.as_str())
            }
        }
    }
}

/// **Every stop the walk makes, in the order the pane paints them.**
///
/// It is the keyboard's cursor track and it is a **query**, derived from the
/// same rows and the same order [`super::render`] draws — so a key cannot walk
/// onto a row a click cannot reach, and cannot walk in an order the glass does
/// not show. A closed server offers its own row and nothing under it; a workspace
/// this seat holds no name for is on neither surface, because no envelope can
/// address it; and a workspace that is not the aimed one offers no conversations,
/// because this seat has asked about none (§4.12) and the glass paints none.
pub fn track(model: &Model) -> Vec<Step> {
    let open = model.server_open();
    let mut rows = Vec::new();
    for server in model.server_rows() {
        let name = server.name().to_owned();
        rows.push(Step::Stand(Stand::Server(name.clone())));
        if open.as_deref() != Some(name.as_str()) {
            continue;
        }
        for workspace in super::ordered(&server.workspaces()) {
            let Some(aim) = server.aim(&workspace) else {
                continue;
            };
            let aimed = model.aim.as_ref() == Some(&aim);
            rows.push(Step::Workspace(aim.clone()));
            if aimed {
                rows.extend(
                    model
                        .rows()
                        .into_iter()
                        .map(|row| Step::Conversation(row.root_id)),
                );
            }
            // `+ new` stands last under every workspace (`super::workspace::begin`).
            rows.push(Step::Stand(Stand::Begin(aim)));
        }
        if let Some(channel) = founds(&server) {
            rows.push(Step::Stand(Stand::Found(channel.to_owned())));
        }
    }
    rows
}

/// **Paint one server's row, and take a click on it.**
///
/// The open one is the chosen row, which is what an accordion has instead of a
/// disclosure triangle: the raise says which one you are inside. The standing
/// one carries the brand rule, and takes the keyboard so the ring is on it and
/// Enter fires it.
///
/// **It carries no `+` any more** (§4.42 item 7): the start is `+ new` under
/// each workspace (`super::workspace::NEW`), the row the conversation would hang from,
/// rather than a glyph at the far edge of a different row. A server with no
/// workspace this seat can address has nothing to begin on, which the composer
/// already said — a start is *a workspace and no conversation*.
pub fn render(ui: &mut egui::Ui, model: &mut Model, server: &Server, open: bool, reveal: bool) {
    let name = server.name().to_owned();
    let standing = model.standing == Some(Stand::Server(name.clone()));
    let seat = theme::paint::row(
        ui,
        &server.header(),
        theme::INK,
        standing.then_some(theme::BRAND),
        open,
        0.0,
        if open { CLOSE_HELP } else { OPEN_HELP },
    );
    if standing && reveal {
        seat.scroll_to_me(None);
    }
    // **Tab and the arrows agree** (bl-2d6b), one row kind over: a Tab that
    // lands on a server's row stands the cursor there, so the next arrow
    // steps off the row the ring is on.
    if seat.gained_focus() {
        model.standing = Some(Stand::Server(name.clone()));
    }
    if seat.clicked() {
        model.open_server(&name);
    }
}

/// **The row that founds a workspace** (§4.42 item 7; bl-673e), standing last
/// under this box's own server beside the workspaces' `+ new` rows.
pub const FOUND: &str = "+ new workspace";
/// **What founding does** (bl-71d0): its own sentence, since what the row
/// begins is a workspace and not a conversation.
pub const FOUND_HELP: &str =
    "founds a workspace on this server: the box asks for its name and what it is for, on one line";
/// What a server's row does, each way.
pub const OPEN_HELP: &str = "opens this server: its workspaces stand beneath";
pub const CLOSE_HELP: &str = "closes this server's list";

/// **`+ new workspace`**: the box asks for `<name> <goal>`
/// ([`Model::begin_founding`]), and the switcher closes on the way.
///
/// **Under this box's own server and no other**: a gesture naming a
/// workspace no entry holds goes down the flat root (REMOTE §8.2), so a name
/// typed under an entry's heading would found it on a different server than
/// the one it was offered under. It stands down where that channel will not
/// dial and under a covering pane, `+ new`'s own rules.
pub fn found(ui: &mut egui::Ui, model: &mut Model, server: &Server, reveal: bool) {
    let Some(channel) = founds(server) else {
        return;
    };
    if model.covered() && !model.switching {
        return;
    }
    let stand = Stand::Found(channel.to_owned());
    if super::workspace::standing_row(ui, model, (FOUND, FOUND_HELP), stand, 0.0, reveal) {
        model.begin_founding(channel);
        model.chose();
    }
}

/// **The channel `+ new workspace` founds on under this server**, where it
/// stands at all — one answer for the paint and the walk.
fn founds(server: &Server) -> Option<&str> {
    // The box's own server leads any fold it is in (`crate::ui::model::fold`).
    let chunk = &server.lead;
    (chunk.channel.name == crate::seat::OWN && !matches!(chunk.held, crate::ui::Held::Unheld(_)))
        .then_some(chunk.channel.name.as_str())
}
