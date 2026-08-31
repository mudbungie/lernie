//! The four structural doors: their usage lines, their arity refusals, and
//! that every word the usage lists can be asked about.

use super::{ASK, ENTRIES, HELP, START, find, table};

/// **A door's usage line is what it prints**, including the one shape a list of
/// envelope field names cannot spell: `help`'s optional argument.
#[test]
fn every_door_prints_the_line_an_operator_types() {
    assert_eq!(START.usage(), "lernie start <workspace> <goal>");
    assert_eq!(ASK.usage(), "lernie ask <envelope>");
    assert_eq!(ENTRIES.usage(), "lernie entries");
    assert_eq!(HELP.usage(), "lernie help [<verb>]");
}

/// **The arity refusal names the word and its grammar**, in the same words the
/// gesture table's does — where all four used to earn *"unrecognised argument"*,
/// the sentence a genuine typo gets, about words the usage lists one screen up.
#[test]
fn an_arity_refusal_names_the_word_and_what_it_takes() {
    assert_eq!(
        ENTRIES.refused(2),
        "`lernie entries` takes 0 argument(s) and got 2 — usage: lernie entries"
    );
    assert_eq!(
        START.refused(1),
        "`lernie start` takes 2 argument(s) and got 1 — usage: lernie start \
         <workspace> <goal>"
    );
    assert_eq!(
        HELP.refused(2),
        "`lernie help` takes at most 1 argument(s) and got 2 — usage: lernie \
         help [<verb>]",
        "the one door whose two arities differ says so"
    );
}

/// Every door is findable by its own word, and nothing else is.
#[test]
fn a_door_is_found_by_its_word_and_a_stranger_is_not() {
    for door in table() {
        assert_eq!(find(door.word), Some(door), "{}", door.word);
    }
    assert_eq!(find("workspaces"), None, "a gesture verb is not a door");
    assert_eq!(find("bogus"), None);
}

/// **The pages are the point** (bl-6bda, bl-81dd): `lernie help <word>` answers
/// for every entry of both lists, where four of eleven used to be refused with
/// *"no verb named `ask`"* — false in the only sense the operator means it, and
/// byte for byte what a typo earns.
#[test]
fn every_word_the_usage_lists_has_a_page() {
    for word in table()
        .iter()
        .map(|door| door.word)
        .chain(crate::verbs::table().iter().map(|verb| verb.word))
    {
        let page = crate::verbs::help::page(word).unwrap_or_else(|why| panic!("{word}: {why}"));
        assert!(
            page.starts_with(&format!("usage: lernie {word}")),
            "{word}: {page}"
        );
    }
    assert!(
        crate::verbs::help::page("bogus")
            .expect_err("a stranger is refused")
            .contains(r#"no word named "bogus""#)
    );
}

/// The usage's own door section is derived from this table, so a door added
/// cannot leave the page behind — the two homes for one prose that made this
/// defect are one home now.
#[test]
fn the_usage_s_door_section_is_this_table() {
    let printed = crate::cli::usage();
    for door in table() {
        assert!(printed.contains(&door.usage()), "{}", door.usage());
        let opening = door
            .detail
            .split_whitespace()
            .take(4)
            .collect::<Vec<&str>>()
            .join(" ");
        assert!(printed.contains(&opening), "{opening}");
    }
}
