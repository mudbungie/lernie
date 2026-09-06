//! **The vocabulary every rendering is built from** — a heading with rows
//! under it, a clause that is there only when its fact is, an age, a tally,
//! and one line of a long text.
//!
//! It is here rather than at each kind for the reason the verb table is data:
//! forty-one renderings written freehand are forty-one chances to spell a
//! listing two ways. **And it is where the branches live.** Every optional
//! field on this surface is rendered by handing it to [`clause`], so the two
//! arms of *present* and *absent* are written once and executed by whatever
//! frame happens to carry each — rather than once per field, where a corpus
//! with one fixture per kind could never reach both.

/// How wide a preview or an excerpt is printed before it is elided. A terminal
/// is the audience and a wrapped row is a row an eye cannot scan; the whole
/// text is one `--json` away.
pub(crate) const BRIEF: usize = 100;

/// **A heading with its rows under it**, or the sentence an empty listing
/// earns.
///
/// The empty arm is a parameter and never a default, because an empty listing
/// is the one moment a reader most needs telling what to do next: a fresh
/// world answers `rows: []` correctly and terminally, and the next act is a
/// fact the seat holds (bl-b00f).
pub(crate) fn listing(head: &str, rows: Vec<String>, empty: &str) -> String {
    if rows.is_empty() {
        return format!("{head}\n{}", indent(empty));
    }
    std::iter::once(head.to_owned())
        .chain(rows.iter().map(|row| indent(row)))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Every line of `text`, moved one step in. Multi-line rows keep their shape.
pub(crate) fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<String>>()
        .join("\n")
}

/// **A clause, present only when its fact is.** The one place an absence is
/// decided on this surface.
pub(crate) fn clause(label: &str, value: Option<&str>) -> Option<String> {
    value.map(|said| format!("{label} {said}"))
}

/// A word, present only when the flag is raised. The other absence, and the
/// same rule: a false flag says nothing rather than saying `false`.
pub(crate) fn when(flag: bool, word: &str) -> Option<String> {
    flag.then(|| word.to_owned())
}

/// **A tally, present only when there is something to count.** `0 waiting` is
/// noise on every row that is fine, and the rows that are not are what a
/// reader is scanning for. The word rides verbatim, for the words that are not
/// nouns — `2 waiting`, `3 deep`, `1 queued`.
pub(crate) fn tally(count: u64, word: &str) -> Option<String> {
    (count > 0).then(|| format!("{count} {word}"))
}

/// **A tally of a countable noun**, which is [`tally`] with the one agreement
/// English asks for. It is a second function rather than a rule inside the
/// first because the first counts adverbs too, and `1 waitings` is worse than
/// the wart it would fix.
pub(crate) fn things(count: u64, singular: &str) -> Option<String> {
    tally(
        count,
        &format!("{singular}{}", if count == 1 { "" } else { "s" }),
    )
}

/// The clauses that are there, in one line. `None` and the empty string both
/// drop out, so a caller may hand over a fact it has not decided about.
pub(crate) fn line(parts: Vec<Option<String>>) -> String {
    parts
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<String>>()
        .join("  ")
}

/// **An age, as a person reads one.** Seconds up to a minute, then minutes,
/// hours and days — one unit, because a roster is scanned rather than read and
/// the second unit has never changed a decision.
pub(crate) fn age(secs: i64) -> String {
    for (bound, unit, per) in [(60, "s", 1), (3_600, "m", 60), (86_400, "h", 3_600)] {
        if secs < bound {
            return format!("{}{unit}", secs / per);
        }
    }
    format!("{}d", secs / 86_400)
}

/// **One line of a long text, elided.** A preview, an excerpt, a goal: each is
/// arbitrary prose from somewhere else, and a terminal listing cannot afford
/// its newlines. The elision is marked, so a reader knows to ask `--json`.
pub(crate) fn brief(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    if flat.chars().count() <= BRIEF {
        return flat;
    }
    format!("{}…", flat.chars().take(BRIEF).collect::<String>())
}

/// A quoted brief — what a preview or a body looks like on a row, where the
/// quotes are what say the prose is somebody else's.
pub(crate) fn quoted(text: &str) -> Option<String> {
    let said = brief(text);
    (!said.is_empty()).then(|| format!("{said:?}"))
}

/// **A head, and a body indented under it when there is one.** The other
/// shape every rendering is made of, beside [`listing`]: a listing is N rows
/// under a heading, and this is one thing said over what it says.
pub(crate) fn line_over(head: &str, body: Option<String>) -> String {
    match body.filter(|said| !said.is_empty()) {
        None => head.to_owned(),
        Some(said) => format!("{head}\n{}", indent(&said)),
    }
}

#[cfg(test)]
mod tests;
