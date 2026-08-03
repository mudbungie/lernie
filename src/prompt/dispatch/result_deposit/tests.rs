//! The terminal deposit's addressing rule (ARCH §2.6): a **reply**
//! answers the last prompter, an **obituary** reports to the dispatcher,
//! and both can address nobody.

use super::*;
use serde_json::json;
use tempfile::TempDir;

const PARENT: &str = "20260101-a1";
const CHILD: &str = "20260101-a1-20260102-b2";
const SIBLING: &str = "20260101-a1-20260102-c3";

/// A worktree carrying the given `messages/` entries, in the order
/// written (name, body).
fn transcript(entries: &[(&str, &str)]) -> TempDir {
    let wt = TempDir::new().unwrap();
    std::fs::create_dir_all(wt.path().join("messages")).unwrap();
    for (name, body) in entries {
        std::fs::write(wt.path().join("messages").join(name), body).unwrap();
    }
    wt
}

/// A delivered prompt's on-disk body (§2.11 frontmatter).
fn prompt(from: &str) -> String {
    format!("---\nfrom: {from}\ndeposited_at: t\n---\ndo it\n")
}

/// A delivered result message's body — a prompt plus the two pinned
/// result fields (§2.6), which is what makes it a return, not a prompt.
fn result(from: &str) -> String {
    format!(
        "---\nfrom: {from}\ndeposited_at: t\nepitaph: final-response\nterminal_ref: sha\n---\nok\n"
    )
}

fn reply_to(wt: &TempDir, agent: &str) -> Option<String> {
    recipient(wt.path(), agent, Epitaph::FinalResponse).unwrap()
}

#[test]
fn terminal_text_joins_text_blocks_and_skips_non_text() {
    let blocks = vec![
        Content::Text("hello ".into()),
        Content::Thinking {
            text: "ignored".into(),
            signature: None,
            id: None,
            encrypted_content: None,
        },
        Content::ToolUse {
            id: "t1".into(),
            name: "bash".into(),
            input: json!({}),
            signature: None,
        },
        Content::Text("world".into()),
    ];
    assert_eq!(terminal_text(&blocks).as_deref(), Some("hello world"));
}

#[test]
fn terminal_text_is_none_when_agent_did_not_speak() {
    let blocks = vec![Content::ToolUse {
        id: "t1".into(),
        name: "bash".into(),
        input: json!({}),
        signature: None,
    }];
    assert_eq!(terminal_text(&blocks), None);
    assert_eq!(terminal_text(&[]), None);
}

#[test]
fn a_reply_answers_the_dispatcher_when_the_dispatch_is_the_last_prompt() {
    // The old parent-addressed rule as this rule's first case: the goal
    // arrives as the dispatcher's message (§2.5), so it is the last
    // prompt until somebody else speaks.
    let wt = transcript(&[
        ("001-20260101-a1.md", &prompt(PARENT)),
        ("002-claude-sonnet-5.json", "[]"),
        ("003-tool.json", "[]"),
    ]);
    assert_eq!(reply_to(&wt, CHILD).as_deref(), Some(PARENT));
}

#[test]
fn a_reply_answers_the_operator_by_depositing_nothing() {
    // The bl-a96a defect: the operator prompted this agent's own
    // conversation, so the answer is read there — no agent inbox is
    // addressed, the same structural no-op a root's deposit takes.
    let wt = transcript(&[
        ("001-20260101-a1.md", &prompt(PARENT)),
        ("002-user.md", &prompt("user")),
    ]);
    assert_eq!(reply_to(&wt, CHILD), None);
}

#[test]
fn a_reply_answers_a_sibling_that_prompted_it() {
    let wt = transcript(&[
        ("001-20260101-a1.md", &prompt(PARENT)),
        ("002-20260101-a1-20260102-c3.md", &prompt(SIBLING)),
    ]);
    assert_eq!(reply_to(&wt, CHILD).as_deref(), Some(SIBLING));
}

#[test]
fn a_returning_childs_result_is_not_a_prompt() {
    // A parent whose child just returned still answers whoever prompted
    // *it*; without the skip every parent would address its own answer
    // to the last child that returned.
    let wt = transcript(&[
        ("001-user.md", &prompt("user")),
        ("002-20260101-a1-20260102-b2.md", &result(CHILD)),
    ]);
    assert_eq!(reply_to(&wt, PARENT), None);
}

#[test]
fn an_agents_own_note_to_itself_is_not_a_prompt() {
    // §2.11 self-messages: replying to one's own inbox would deposit
    // into the very inbox whose delivery produced the reply.
    let wt = transcript(&[
        ("001-20260101-a1.md", &prompt(PARENT)),
        ("002-20260101-a1-20260102-b2.md", &prompt(CHILD)),
    ]);
    assert_eq!(reply_to(&wt, CHILD).as_deref(), Some(PARENT));
}

#[test]
fn a_reply_with_no_surviving_prompt_falls_back_to_the_dispatcher() {
    // Compaction can squash the dispatch message out of the record
    // (§2.6); the id still carries the one sender the branch's own
    // existence records. A root has neither and deposits nothing.
    let empty = transcript(&[
        ("001-claude-sonnet-5.json", "[]"),
        ("noseq.md", "x"),
        ("abc-user.md", "x"),
        ("001-.md", "x"),
    ]);
    assert_eq!(reply_to(&empty, CHILD).as_deref(), Some(PARENT));
    assert_eq!(reply_to(&empty, PARENT), None);
    // A worktree with no `messages/` at all is the same empty input.
    let bare = TempDir::new().unwrap();
    assert_eq!(
        recipient(bare.path(), CHILD, Epitaph::FinalResponse).unwrap(),
        Some(PARENT.to_string())
    );
}

#[test]
fn an_obituary_reports_to_the_dispatcher_whoever_prompted_last() {
    // A stop, an exhausted ceiling and a death are facts about the tree,
    // not answers: the operator's prompt does not redirect them.
    let wt = transcript(&[("001-user.md", &prompt("user"))]);
    for epitaph in [Epitaph::Stopped, Epitaph::BudgetExhausted, Epitaph::Died] {
        assert_eq!(
            recipient(wt.path(), CHILD, epitaph).unwrap().as_deref(),
            Some(PARENT),
            "{epitaph:?} is an obituary"
        );
        assert_eq!(recipient(wt.path(), PARENT, epitaph).unwrap(), None);
    }
}

#[test]
fn a_messages_path_that_is_not_a_directory_surfaces() {
    // Not the NotFound arm (an empty branch), so it is an error, not an
    // empty input.
    let wt = TempDir::new().unwrap();
    std::fs::write(wt.path().join("messages"), "not a directory").unwrap();
    assert!(recipient(wt.path(), CHILD, Epitaph::FinalResponse).is_err());
}

#[test]
fn an_unreadable_transcript_entry_surfaces_rather_than_misaddressing() {
    // The read is the derivation's evidence; a directory in its place is
    // an I/O error, never a silent "nobody prompted me".
    let wt = transcript(&[]);
    std::fs::create_dir(wt.path().join("messages/001-user.md")).unwrap();
    assert!(recipient(wt.path(), CHILD, Epitaph::FinalResponse).is_err());
}
