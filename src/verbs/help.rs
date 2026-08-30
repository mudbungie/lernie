//! **Help, and its subject is the interface rather than the world.**
//!
//! That is the whole reason it is answered here. Every other verb is a question
//! about a world this process cannot see, so it needs a channel, an engine that
//! is up, and material an operator carried by hand. *"What does this verb
//! take?"* is a question about **this binary**, and a binary that could not
//! answer it without a server would be a binary an operator cannot learn to use
//! until the hard part already works.
//!
//! So there is no `help` gesture, no dial and no wait — and there is exactly
//! one help. `lernie help`, `lernie --help` and `lernie -h` all print the
//! usage, whose verb section is [`roster`]; `lernie help <verb>` prints that
//! verb's [`page`]. Two spellings of one subject would be two texts to keep in
//! step.

use super::{Verb, find, table};

/// The verb section of the usage: every verb's own line, aligned.
///
/// The alignment is computed from the widest usage line rather than fixed, so a
/// verb added tomorrow cannot leave the column behind — the same rule the usage
/// line itself follows.
pub fn roster() -> String {
    let rows: Vec<(String, &str)> = table()
        .iter()
        .map(|verb| (verb.usage(), verb.summary))
        .collect();
    let widest = rows.iter().map(|(usage, _)| usage.len()).max().unwrap_or(0);
    rows.iter()
        .map(|(usage, summary)| format!("  {usage:widest$}   {summary}"))
        .collect::<Vec<String>>()
        .join("\n")
}

/// One verb's page: what to type, what it is for, and what to know first.
///
/// A word that is not a verb refuses **naming it** and pointing at the roster,
/// because the operator who typed it is the one who needs the list.
pub fn page(word: &str) -> Result<String, String> {
    let Some(verb) = find(word) else {
        return Err(format!(
            "no verb named {word:?} — `lernie help` lists every one"
        ));
    };
    Ok(rendered(&verb))
}

/// The page proper: the usage line, the summary under it, then the detail
/// wrapped to a width a terminal holds.
fn rendered(verb: &Verb) -> String {
    format!(
        "usage: {}\n\n{}\n\n{}",
        verb.usage(),
        verb.summary,
        wrapped(verb.detail)
    )
}

/// The detail, folded at a readable width. The table stores one paragraph and
/// this is where it becomes lines: a stored line break would be a second fact
/// about the same prose, and it would be wrong on every terminal but one.
fn wrapped(text: &str) -> String {
    const WIDTH: usize = 72;
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= WIDTH => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests;
