//! **The one dispatch**: an answer, and the rendering it is.
//!
//! It is the census read in the other direction — [`crate::reply::Reply`] says
//! what an engine can answer, and this says what each of those looks like — so
//! a kind added there is a match arm missing here, which is the compiler
//! saying the rendering half of a pane was not built. That is the same
//! discipline the reply vocabulary itself holds: a kind nothing renders is a
//! kind nobody has to carry.
//!
//! The arms delegate by family and the families are the panes', not the wire's:
//! what a wall holds, what one conversation's records are, what the balls and
//! the fleet are doing, and what an ACT came back saying.

use crate::reply::Reply;

use super::{acts, chat, policy, reads, records, step, tasks, walls};

/// What one answer looks like to a person.
pub(super) fn answer(reply: &Reply) -> String {
    match reply {
        Reply::Workspaces(roster) => walls::workspaces(roster),
        Reply::Conversations(rows) => walls::conversations(rows),
        Reply::Attention(rows) => walls::attention(rows),
        Reply::Transcript(transcript) => chat::transcript(transcript),
        Reply::Follow(stream) => walls::follow(stream),
        Reply::Agent(agent) => chat::agent(agent),

        Reply::Steps(steps) => records::steps(steps),
        Reply::Step(one) => step::step(one),
        Reply::Files(files) => records::files(files),
        Reply::Inbox(rows) => records::inbox(rows),
        Reply::Rail(rail) => records::rail(rail),
        Reply::Governing(governing) => records::governing(governing),
        Reply::Roles(rows) => policy::roles(rows),
        Reply::Config(config) => policy::config(config),
        Reply::Lineages(rows) => policy::lineages(rows),
        Reply::Clients(rows) => policy::clients(rows),

        Reply::Balls(rows) => tasks::balls(rows),
        Reply::Board(board) => tasks::board(board),
        Reply::WorkspaceBalls(rows) => tasks::workspace_balls(rows),
        Reply::Marks { branch } => format!("tasks tracked on {branch}"),
        Reply::Science(rows) => tasks::science(rows),
        Reply::Work(rows) => tasks::work(rows),
        Reply::Ops(rows) => reads::ops(rows),
        Reply::Found(found) => reads::found(found),
        Reply::Help(rows) => reads::help(rows),

        Reply::Outcome(outcome) => acts::outcome(outcome),
        Reply::Prepared(prepared) => acts::prepared(prepared),
        Reply::Fanned(candidates) => acts::fanned(candidates),
        Reply::Enrolled(enrolled) => acts::enrolled(enrolled),
        Reply::Login(signin) => acts::login(signin),
        Reply::Providers(rows) => acts::providers(rows),
        Reply::Models(rows) => acts::models(rows),
        Reply::Answered {
            tool,
            tool_use,
            verdict,
            advanced,
        } => acts::answered(tool, tool_use, verdict, *advanced),
        Reply::Delivered {
            base,
            target,
            source,
            commit,
        } => acts::delivered(base, target, source.as_deref(), commit.as_deref()),

        // **The receipts**, which carry one fact or none. Each is a sentence
        // and not a row, because there is no listing to be a row of — what
        // changed is on a read the operator takes next, and a receipt that
        // restated it would be this end predicting a listing
        // ([`crate::reply::Reply::Nudged`] on why).
        Reply::Started { conversation } => format!("started {conversation}"),
        Reply::Nudged => "nudged — the advance is running".to_owned(),
        Reply::Flagged => "flagged — it is on the queue now".to_owned(),
        Reply::Acked => "the trail's watermark is where you are".to_owned(),
        Reply::TrailCleared => "the trail is truncated".to_owned(),
        Reply::Retired { discarded } => acts::retired(*discarded),
        Reply::Floored { standing } => acts::floored(*standing),
        Reply::Armed(standing) => acts::armed(*standing),
    }
}

#[cfg(test)]
mod tests;
