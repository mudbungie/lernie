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

use super::doors;
use super::{find, table};

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

/// **The doors' section of the usage**: each word, then its paragraph under it.
///
/// It is derived from [`doors`]'s own table for the same reason [`roster`] is
/// derived from the gesture table — the usage used to carry these four
/// paragraphs by hand, beside a `lernie help <verb>` that could not reach any
/// of them (bl-6bda), which is two homes for one prose and a promise kept for
/// seven words out of eleven.
pub fn doors() -> String {
    doors::table()
        .iter()
        .map(|door| format!("  {}\n{}", door.usage(), indented(door.detail)))
        .collect::<Vec<String>>()
        .join("\n\n")
}

/// One word's page: what to type, what it is for, and what to know first.
///
/// **Every word the usage lists has one**, gesture row and structural door
/// alike: a page offered over a list is a page the list's every entry must
/// have, and the four that did not were refused in the bytes a typo earns — on
/// the one surface whose whole job is to answer with nothing provisioned.
///
/// A word that is neither refuses **naming it** and pointing at the roster,
/// because the operator who typed it is the one who needs the list.
pub fn page(word: &str) -> Result<String, String> {
    if let Some(verb) = find(word) {
        return Ok(rendered(&verb.usage(), verb.summary, verb.detail));
    }
    if let Some(door) = doors::find(word) {
        return Ok(rendered(&door.usage(), door.summary, door.detail));
    }
    Err(format!(
        "no word named {word:?} — `lernie help` lists every one"
    ))
}

/// The page proper: the usage line, the summary under it, then the detail
/// wrapped to a width a terminal holds. **One renderer**, because a gesture's
/// page and a door's are one page with two sources.
fn rendered(usage: &str, summary: &str, detail: &str) -> String {
    format!("usage: {usage}\n\n{summary}\n\n{}", wrapped(detail))
}

/// The same fold, hung four spaces under a word in the usage's door section.
fn indented(text: &str) -> String {
    wrapped(text)
        .lines()
        .map(|line| format!("      {line}"))
        .collect::<Vec<String>>()
        .join("\n")
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
