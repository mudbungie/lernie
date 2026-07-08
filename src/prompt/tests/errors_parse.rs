//! Parse-level failures on the brazen `v=1` event wire (ARCH §4.4).
//!
//! A `response.json` line that is not a decodable `brazen::Event`
//! surfaces as [`Error::AdapterJson`] — the harness will not proceed on
//! a stream it cannot parse. (Unknown *event types* are tolerated via
//! brazen's forward-compat `Other`; these cases are structurally
//! undecodable, not merely unknown.)

use super::fixtures::*;
use crate::prompt::Error;

#[test]
fn run_rejects_undecodable_event_lines() {
    let cases: &[(&str, &[u8])] = &[
        ("non-object JSON", b"[1,2,3]\n"),
        ("object with no type tag", b"{\"unexpected\":\"shape\"}\n"),
        (
            "error event missing kind",
            b"{\"type\":\"error\",\"message\":\"boom\"}\n",
        ),
        (
            "error event missing message",
            b"{\"type\":\"error\",\"kind\":\"transport\"}\n",
        ),
    ];
    for (label, body) in cases {
        let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
        let adapter = StubAdapter::happy(body);
        let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
        assert!(
            matches!(err, Error::AdapterJson(_)),
            "{label}: expected AdapterJson, got {err:?}"
        );
    }
}
