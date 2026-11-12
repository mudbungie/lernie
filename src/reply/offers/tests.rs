//! The offer's replies, read off frames the server's own corpus spells.

use super::{OfferRow, Offered, clock};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// `offer` answers the URL and the expiry, and nothing else.
#[test]
fn a_minted_offer_reads_as_its_url_and_expiry() {
    let frame = json!({"ok": true, "kind": "offered",
        "url": "https://engine.invalid/yog/register/ABCD2345", "expires": 1_700_000_600});
    assert_eq!(
        read(&frame),
        Read::Answer(Reply::Offered(Offered {
            url: "https://engine.invalid/yog/register/ABCD2345".to_owned(),
            expires: 1_700_000_600,
        }))
    );
    let short = json!({"ok": true, "kind": "offered", "url": "https://engine.invalid/x"});
    assert!(matches!(read(&short), Read::Unreadable(why) if why.contains("expires")));
}

/// A row's word is absent until a device has claimed, and only a proved row
/// can be approved.
#[test]
fn a_pending_offer_reads_with_its_state_and_its_word_when_it_has_one() {
    let frame = json!({"ok": true, "kind": "offers", "rows": [
        {"name": "phone-1", "grade": "operator", "state": "open", "expires": 1_700_000_600},
        {"name": "builder", "grade": "foot", "state": "proved", "expires": 1_700_000_600,
         "word": "2LFTUR"},
    ]});
    let Read::Answer(Reply::Offers(rows)) = read(&frame) else {
        panic!("offers reads as an answer");
    };
    let open = rows.first().expect("the open row");
    assert_eq!((open.word.as_deref(), open.proved()), (None, false));
    assert_eq!(open.line(), "phone-1  operator  open");
    let proved = rows.get(1).expect("the proved row");
    assert_eq!(
        (proved.word.as_deref(), proved.proved()),
        (Some("2LFTUR"), true)
    );
    assert_eq!(proved.line(), "builder  foot  proved  word 2LFTUR");
    let wrong = json!({"ok": true, "kind": "offers", "rows": ["phone"]});
    assert!(matches!(read(&wrong), Read::Unreadable(why) if why.contains("not a JSON object")));
    let short = json!({"ok": true, "kind": "offers", "rows": [{"name": "phone"}]});
    assert!(matches!(read(&short), Read::Unreadable(_)));
}

/// The two receipts carry nothing.
#[test]
fn approving_and_declining_answer_bare_receipts() {
    for (kind, reply) in [("approved", Reply::Approved), ("declined", Reply::Declined)] {
        assert_eq!(
            read(&json!({"ok": true, "kind": kind})),
            Read::Answer(reply)
        );
    }
}

/// A moment reads as the clock time it is, in UTC.
#[test]
fn a_moment_reads_as_a_clock_time() {
    assert_eq!(clock(1_700_000_600), "22:23 UTC");
    assert_eq!(clock(0), "00:00 UTC");
    assert_eq!(clock(86_399), "23:59 UTC");
    let row = OfferRow {
        name: "n".to_owned(),
        grade: "foot".to_owned(),
        state: "later".to_owned(),
        expires: 0,
        word: None,
    };
    assert!(
        !row.proved(),
        "a state this build has not met approves nothing"
    );
}
