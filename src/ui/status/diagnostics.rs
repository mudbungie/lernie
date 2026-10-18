//! **The conversation's diagnostics** (DESIGN §4.42, item 11): what the rule
//! says after the selectors, every one a fact about THIS conversation —
//! `/plan`'s arming, the state slot, the tokens in and out, what it cost and
//! how full its context is. The last three are the `agent` row's own figures,
//! so a window that has not been answered about the selection says none.

use super::run;
use crate::reply::agent::Agent;
use crate::ui::{Model, Notice, theme};

/// The state slot's word where nothing is selected and nothing is heard.
pub const READY: &str = "ready";
/// **The mode the next start is armed in**, first among the diagnostics while
/// `/plan` holds it (§4.42 item 3; bl-6f3f).
pub const PLAN: &str = "plan";

/// **The state slot**: the notice, else why the aimed server will not dial,
/// else the selected conversation's state and age, else `ready`.
pub fn state(model: &Model) -> super::Run {
    if let Some(notice) = &model.notice {
        let ink = match notice {
            Notice::Said(_) => theme::accent(theme::State::Annotation),
            _ => theme::accent(theme::State::Error),
        };
        return run(notice.line(), ink);
    }
    match down(model) {
        Some(why) => run(why, theme::accent(theme::State::Error)),
        None => calm(model),
    }
}

/// **Why the server this conversation is on will not dial**, where it will
/// not (bl-4cac): the one dial-state fact about THIS conversation, so it
/// stays on the rule while the seat-wide count of servers left it.
pub fn down(model: &Model) -> Option<String> {
    let aim = model.aim.as_ref()?;
    model
        .roster
        .iter()
        .find(|chunk| chunk.channel.name == aim.channel)
        .filter(|chunk| matches!(chunk.held, crate::ui::Held::Unheld(_)))
        .and_then(|chunk| chunk.held.said())
}

/// **The state slot with no notice in it** — what it says under a notice too
/// long to stand there (bl-7e3e; [`super::render`]).
pub fn calm(model: &Model) -> super::Run {
    match super::selected(model) {
        Some(row) => run(
            format!(
                "{} · {}",
                row.state.label(),
                crate::ui::convs::age(row.age_secs)
            ),
            theme::state_ink(&row.state),
        ),
        None => run(READY, theme::INK_WEAK),
    }
}

/// **The menu rows a sentence names** (bl-7a77): every `/word` in it that
/// is a row this seat offers right now, once each in the order said — so a
/// refusal saying *sign in first … /config brazen* names `/config`, and the
/// remedy can be a click as well as a word. The server words the remedy and
/// the menu decides what is a door ([`super::door`]); nothing here names a
/// verb.
pub fn named(model: &Model, sentence: &str) -> Vec<crate::ui::slash::Row> {
    let mut out: Vec<crate::ui::slash::Row> = Vec::new();
    for word in sentence.split_whitespace() {
        let Some(word) = word.strip_prefix(crate::ui::slash::LEAD) else {
            continue;
        };
        let word = word.trim_end_matches(|c: char| !c.is_alphanumeric());
        if let Some(row) = super::door(model, word)
            && !out.contains(&row)
        {
            out.push(row);
        }
    }
    out
}

/// **The doors the notice names**, as runs in the brand beside it on the
/// rule, each firing its row ([`named`]).
pub fn doors(model: &Model) -> Vec<super::Run> {
    let Some(notice) = &model.notice else {
        return Vec::new();
    };
    named(model, &notice.line())
        .into_iter()
        .map(|row| super::Run {
            door: Some(super::Door::Row(row)),
            ..run(
                format!("{}{}", crate::ui::slash::LEAD, row.word),
                theme::BRAND,
            )
        })
        .collect()
}

/// **A count as the rule says it**: whole below ten thousand, then `k` and
/// `M` to one decimal.
pub fn count(n: u64) -> String {
    match n {
        0..10_000 => n.to_string(),
        10_000..1_000_000 => format!("{}.{}k", n / 1000, n % 1000 / 100),
        _ => format!("{}.{}M", n / 1_000_000, n % 1_000_000 / 100_000),
    }
}

/// **The tokens in and out**, and the cache's read and write where either is
/// non-zero.
pub fn tokens(row: &Agent) -> Vec<String> {
    let spent = &row.spend.tokens;
    let mut out = vec![format!(
        "{} in · {} out",
        count(spent.input),
        count(spent.output)
    )];
    if spent.cache_read + spent.cache_write > 0 {
        out.push(format!(
            "cache {} read · {} write",
            count(spent.cache_read),
            count(spent.cache_write)
        ));
    }
    out
}

/// **The diagnostics, in order.**
pub fn diagnostics(model: &Model) -> Vec<super::Run> {
    let mut out = Vec::new();
    if model.plan {
        out.push(run(PLAN, theme::BRAND));
    }
    out.push(state(model));
    out.extend(doors(model));
    let Some(row) = model.agent() else {
        return out;
    };
    for said in tokens(row) {
        out.push(run(said, theme::INK_WEAK));
    }
    // **The money in §4.41's words**, and nothing where no rate priced it —
    // never `$0`, which would be a claim the server did not make.
    if let Some(cost) = &row.spend.cost {
        out.push(run(cost.said(), theme::INK_WEAK));
    }
    if let Some(full) = &row.context {
        out.push(run(format!("context {}%", full.percent), theme::INK_WEAK));
    }
    out
}
