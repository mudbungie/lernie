//! **An enrollment, between the control that opened it and the symbol it ends
//! at** (yog's `docs/REMOTE.md` §8.4; DESIGN §3).
//!
//! Two things this holds that nothing else in the window holds, and both are
//! the reason it is its own type.
//!
//! # It holds a secret, and holding it is the whole of what the seat does
//!
//! [`Shown`] carries a **private key for a box that does not exist yet**. The
//! seat never writes it: not to its state root, not to a cache, not to a log
//! line. It lives here while a symbol is on screen and is dropped with the
//! pane — [`Model::close_enrollment`](super::Model::close_enrollment) is what
//! drops it, and the operator reaches that through a control.
//!
//! # The symbol is cached because a frame may not compute one
//!
//! Every other thing in this model is a value a frame reads and paints.
//! Encoding a symbol is Reed-Solomon over about sixteen hundred bytes and eight
//! masked candidates scored four ways — microseconds, but microseconds *per
//! frame*, forever, on a window that repaints on a beat. So it is computed once
//! where the answer is filed and held beside the material it came from. That is
//! not a second representation of one fact: it is a derivation whose only
//! alternative is doing it again sixty times a second, which is the one thing
//! `crate::ui`'s own rule forbids a frame to do.

use crate::qr::Symbol;
use crate::reply::enrolled::Enrolled;

use super::Aim;

/// **Which grade a new box's leaf is minted at** (REMOTE §4.2).
///
/// Two arms rather than a typed string, because this is a *choice a control
/// offers* — the window paints one button per arm, and an arm added is a button
/// added. The wire word is [`Grade::word`], which is the one place either
/// spelling is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grade {
    /// A seat: it asks, acts and paints. The default, because it is the grade
    /// an operator enrolling their own next device wants.
    #[default]
    Operator,
    /// A tool host: it executes, and reads nothing (REMOTE §5.4).
    Foot,
}

impl Grade {
    /// The wire's own word for it.
    pub fn word(self) -> String {
        match self {
            Self::Operator => "operator",
            Self::Foot => "foot",
        }
        .to_owned()
    }

    /// Both, in the order the control paints them.
    pub fn both() -> Vec<Self> {
        vec![Self::Operator, Self::Foot]
    }
}

/// **The material, and the picture of it.** What the pane paints once the
/// engine has answered.
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    /// The three fields that are not secret, as one line.
    pub caption: String,
    /// The symbol carrying all six.
    pub symbol: Symbol,
}

impl Shown {
    /// Draw the material, or say why it will not fit one symbol.
    ///
    /// The ceiling is version 40 at correction level M and REMOTE §8.4 measures
    /// the envelope well inside it, so the `Err` is a recipe that moved — an
    /// RSA key, a longer chain — rather than a picture that could have been
    /// drawn smaller.
    pub fn of(material: &Enrolled) -> Result<Self, String> {
        Symbol::encode(material.envelope().as_bytes())
            .map(|symbol| Self {
                caption: material.caption(),
                symbol,
            })
            .map_err(|too_long| {
                format!("the engine's material will not fit one symbol: {too_long}")
            })
    }
}

/// **An enrollment in flight**: what it is aimed at, what the operator has
/// chosen, and the answer once there is one.
#[derive(Debug, Clone, PartialEq)]
pub struct Enrolling {
    /// The wall it enrolls into — the address a gesture must carry, which is
    /// what makes the pane composable without a second lookup.
    pub aim: Aim,
    /// The new box's name, as typed. The engine is the authority on whether it
    /// is a legal one (REMOTE §4.1: one path component, never `local`) and
    /// refuses in band, so nothing here judges it — a seat that judged it would
    /// be a second authority for a rule it does not own.
    pub name: String,
    /// The grade its leaf is minted at.
    pub grade: Grade,
    /// **Whether the act has been spent**, which nothing else can answer: the
    /// outbox a frame composes into is drained by another thread within the
    /// beat, so a pane reading it back would see the gesture for one frame and
    /// then see nothing. It is the record that the operator has already asked,
    /// and it is what stops a second click minting a second box.
    pub posted: bool,
    /// The material and its picture, once the engine has answered.
    pub shown: Option<Shown>,
}

impl Enrolling {
    /// A fresh enrollment aimed at one wall, with nothing chosen but the
    /// default grade.
    pub fn at(aim: Aim) -> Self {
        Self {
            aim,
            name: String::new(),
            grade: Grade::default(),
            posted: false,
            shown: None,
        }
    }

    /// **Whether the act can be spent yet.** A name is the one thing the
    /// operator has to supply, and an empty one would spend a round trip to
    /// learn what this end already knows. Spent once, it is not ready again:
    /// the engine refuses a second enrollment under one name, but the honest
    /// place to stop a double click is the control that made it.
    pub fn ready(&self) -> bool {
        !self.name.trim().is_empty() && !self.posted && self.shown.is_none()
    }

    /// Whether the engine has been asked and has not yet answered.
    pub fn minting(&self) -> bool {
        self.posted && self.shown.is_none()
    }

    /// The gesture this enrollment composes, built from the same row
    /// `lernie enroll` spends.
    pub fn gesture(&self) -> serde_json::Value {
        crate::verbs::enroll(
            self.aim.address.clone(),
            self.name.trim().to_owned(),
            self.grade.word(),
        )
    }
}
