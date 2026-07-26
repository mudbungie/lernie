//! **The adapter's stderr lands beside the model call (§2.3).** Every
//! attempt's captured stderr appends to `stderr.log` in the same step
//! directory — empty on an ordinary run, since brazen speaks its
//! failures in-band on stdout (§4.4). Bytes there mean the adapter died
//! *outside* the contract with an empty stream: the shape a bare
//! [`crate::prompt::Error::AdapterHalfStream`] misreports as a
//! mid-stream kill, so that error quotes the failing attempt's stderr
//! tail. It stays quiet under a stop (§2.9), needing no flag of its
//! own: a stop-pending half-stream is the *expected* signature, and the
//! caller's §2.9 check point discards the outcome unrendered.

/// How much of an attempt's stderr the half-stream error quotes — big
/// enough for a config-parse complaint, small enough that an error line
/// never becomes a log dump. The whole capture is on disk.
pub(super) const TAIL_CHARS: usize = 400;

/// The trailing [`TAIL_CHARS`] characters of an attempt's stderr,
/// newlines flattened so the error stays one line, with a leading `…`
/// when there is more on disk. `(empty)` when the adapter said nothing
/// — itself diagnostic: an empty stderr with an empty stream is a
/// genuine mid-stream kill.
pub(super) fn tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "(empty)".to_string();
    }
    let start = trimmed
        .char_indices()
        .rev()
        .take(TAIL_CHARS)
        .last()
        .map_or(0, |(i, _)| i);
    let tail = trimmed[start..].replace('\n', " | ");
    if start > 0 {
        format!("…{tail}")
    } else {
        tail
    }
}
