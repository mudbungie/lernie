//! The offer's four replies, as the command line prints them (REMOTE §15.1).

use super::shapes::says;
use crate::render::rendered;
use serde_json::json;

/// **A minted offer is the URL as text and as a symbol, with its expiry in
/// words and what happens next** (REMOTE §15.1) — and a URL too long for a
/// symbol is still printed, with the reason there is no picture.
#[test]
fn a_minted_offer_prints_its_url_its_symbol_and_its_expiry() {
    let url = "https://engine.invalid/yog/register/ABCD2345";
    let said = rendered(&json!({"ok": true, "kind": "offered", "url": url,
                                "expires": 1_700_000_600}));
    let symbol = crate::qr::Symbol::encode(url.as_bytes())
        .expect("a URL fits a symbol")
        .block();
    for part in [
        url,
        symbol.as_str(),
        "It works once, and only until 22:23 UTC",
        "`lernie offers <workspace>`",
        "`lernie approve <workspace> <name>`",
    ] {
        assert!(said.contains(part), "{part:?}:\n{said}");
    }
    let long = rendered(
        &json!({"ok": true, "kind": "offered", "url": "x".repeat(4000),
                                "expires": 0}),
    );
    assert!(
        long.contains("(no symbol: the URL will not fit one"),
        "{long}"
    );
}

/// **The pending offers, one line each**: an unclaimed one says when its
/// code expires, a claimed one its word, and a proved one that it can be
/// approved. The two verdicts are a sentence each.
#[test]
fn the_pending_offers_and_the_two_verdicts_render() {
    says(
        &json!({"ok": true, "kind": "offers", "rows": [
            {"name": "phone-1", "grade": "operator", "state": "open", "expires": 1_700_000_600},
            {"name": "tablet", "grade": "operator", "state": "claimed",
             "expires": 1_700_000_600, "word": "K7M2QX"},
            {"name": "builder", "grade": "foot", "state": "proved",
             "expires": 1_700_000_600, "word": "2LFTUR"},
        ]}),
        &[
            "phone-1  operator  open  code expires  22:23 UTC",
            "tablet  operator  claimed  word K7M2QX",
            "builder  foot  proved  word 2LFTUR  — can be approved",
        ],
    );
    says(
        &json!({"ok": true, "kind": "offers", "rows": []}),
        &["no offer is pending in this workspace"],
    );
    says(
        &json!({"ok": true, "kind": "approved"}),
        &["approved — that device is seated"],
    );
    says(
        &json!({"ok": true, "kind": "declined"}),
        &["declined — that offer is void"],
    );
}
