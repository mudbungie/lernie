//! The model call: build a typed canonical request, exec `bz` once per
//! attempt, own the retry loop (ARCH §4.4, §2.10, §3.5).
//!
//! brazen never retries — each `bz` process performs exactly one HTTP
//! round-trip (§4.4). The harness owns the retry loop: on an in-band
//! `Error` event whose kind is retryable ([`CanonicalError::retryable`],
//! the linked crate's single home for the fact — never re-derived), it
//! re-invokes `bz` with the *identical* request (the assembler is
//! deterministic from the step's recorded commit, so no drift) up to the
//! `workflow.yaml` attempt cap, sleeping the backoff between attempts.
//!
//! **Fd held open for the whole model call (§3.5).** The `response.json`
//! fd is opened once at the first attempt and held across *every*
//! attempt and *every* backoff sleep — closed only at step resolution.
//! fd-open is the single `in_flight` signal, so a mid-retry `Error`
//! segment never reads as `failed` while the loop is still pending.
//! Each attempt's stdout is appended verbatim as one segment; the last
//! segment is authoritative (§4.4).

use super::assembler::{Assembler, AssemblyError, Completion, SegmentOutcome};
use super::staging::{StagingWriter, staging_path_for};
use crate::config::RetryConfig;
use crate::prompt::Error;
use crate::prompt::adapter::AdapterRunner;
use brazen::{CanonicalRequest, Content, EVENT_SCHEMA_VERSION, Message, Tool};
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

/// Injected sleep so the retry backoff is real in production and a
/// no-op in tests (the retry *logic* does not depend on wall time).
pub trait Sleeper {
    fn sleep(&self, dur: Duration);
}

/// Production [`Sleeper`] — blocks the calling thread.
#[derive(Debug, Clone, Copy)]
pub struct RealSleeper;

impl Sleeper for RealSleeper {
    fn sleep(&self, dur: Duration) {
        std::thread::sleep(dur);
    }
}

/// Everything the retry loop needs beyond the request itself.
pub(super) struct ModelCall<'a> {
    pub(super) adapter: &'a dyn AdapterRunner,
    pub(super) sleeper: &'a dyn Sleeper,
    pub(super) binary: &'a OsString,
    pub(super) provider_row: &'a str,
    pub(super) retry: RetryConfig,
    /// True under an `adapter:` override (§4.2): the version guard is
    /// skipped and the in-band `MessageStart.v == EVENT_SCHEMA_VERSION`
    /// handshake governs the completed segment instead (§4.4).
    pub(super) expect_handshake: bool,
}

/// Build a typed [`CanonicalRequest`] (§4.4 "the vocabulary is linked"):
/// building the struct directly makes brazen's fail-open `extra` map
/// unreachable. `stream` is left `None` — streaming is brazen's default
/// and lernie never overrides it (§4.4). `tools` carries the role's
/// composed toolset (§3.3 — the schemas the model is told it may call);
/// an empty vec is "no tools declared/available".
pub(super) fn build_request(
    model_id: &str,
    system: &str,
    messages: Vec<Message>,
    tools: Vec<Tool>,
    max_tokens: u32,
) -> CanonicalRequest {
    CanonicalRequest {
        model: model_id.to_string(),
        system: Some(vec![Content::Text(system.to_string())]),
        messages,
        tools,
        max_tokens: Some(max_tokens),
        ..CanonicalRequest::default()
    }
}

/// Drive one model call to resolution: `bz --json --provider <row>` per
/// attempt, request on stdin, each attempt's stdout appended verbatim to
/// `response_path` as one segment. Returns the assembled [`Completion`]
/// on success; a non-retryable / budget-exhausted `Error`, a half-stream
/// kill, or a malformed event surfaces as a harness [`Error`].
pub(super) fn run(
    call: &ModelCall<'_>,
    request_bytes: &[u8],
    response_path: &Path,
) -> Result<Completion, Error> {
    if let Some(parent) = response_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // One fd, held across every attempt and backoff sleep (§3.5). The
    // staging sink (§2.3) is the second stream off the same pass — the
    // assistant transcript entry under construction, sealed and renamed
    // by the caller once the call settles complete.
    let mut response_file = File::create(response_path)?;
    let mut staging = StagingWriter::create(&staging_path_for(response_path))?;
    let args = ["--json", "--provider", call.provider_row];
    let max = call.retry.max_attempts.max(1);
    let mut attempt = 1;
    loop {
        staging.begin_segment();
        let outcome = run_attempt(call, &args, request_bytes, &mut response_file, &mut staging)?;
        match outcome {
            SegmentOutcome::Complete(completion) => {
                check_handshake(call.expect_handshake, &completion)?;
                staging.seal()?;
                drop(response_file);
                return Ok(completion);
            }
            SegmentOutcome::Failed(err) => {
                // §4.4 segment authority: an `Error`-terminated segment
                // contributes nothing — truncate its blocks from staging.
                staging.truncate_segment()?;
                if err.retryable() && attempt < max {
                    call.sleeper.sleep(call.retry.backoff.delay(attempt));
                    attempt += 1;
                    continue;
                }
                drop(response_file);
                return Err(Error::AdapterError {
                    kind: format!("{:?}", err.kind),
                    message: err.message,
                });
            }
            SegmentOutcome::HalfStream => {
                // Killed mid-stream (§2.9): nothing settled, so staging
                // is left as debris the step's re-run overwrites (§2.3).
                drop(response_file);
                return Err(Error::AdapterHalfStream);
            }
        }
    }
}

/// One `bz` attempt: tee every stdout line to the open `response_file`
/// (as a segment), stream content into the `staging` sink (§2.3), and
/// fold it into a fresh [`Assembler`] (the diagnostic in-memory shape).
/// A malformed event line — or a tool-use block whose `json_delta` does
/// not parse — surfaces as [`Error::AdapterJson`].
fn run_attempt(
    call: &ModelCall<'_>,
    args: &[&str],
    request_bytes: &[u8],
    response_file: &mut File,
    staging: &mut StagingWriter,
) -> Result<SegmentOutcome, Error> {
    let mut assembler = Assembler::new();
    let mut feed_err: Option<serde_json::Error> = None;
    let mut staging_err: Option<Error> = None;
    call.adapter
        .run(call.binary, args, request_bytes, &mut |line| {
            response_file.write_all(line)?;
            response_file.write_all(b"\n")?;
            if feed_err.is_none() && staging_err.is_none() {
                match serde_json::from_slice::<brazen::Event>(line) {
                    Ok(event) => match staging.feed(&event) {
                        Ok(()) => assembler.feed(event),
                        Err(e) => staging_err = Some(e),
                    },
                    Err(e) => feed_err = Some(e),
                }
            }
            Ok(())
        })
        .map_err(Error::AdapterSpawn)?;
    if let Some(e) = feed_err {
        return Err(Error::AdapterJson(e));
    }
    if let Some(e) = staging_err {
        return Err(e);
    }
    // Both sinks parse each tool_use block's `json_delta`: staging at the
    // block's `content_stop` (surfaced as `staging_err` above), the
    // assembler when it finalizes every started block here — the latter
    // catches a bad parse the stream never `content_stop`'d. Same fact,
    // same `AdapterJson`; the assembler path retires with the accumulator
    // (bl-26cb).
    assembler
        .into_outcome()
        .map_err(|AssemblyError(e)| Error::AdapterJson(e))
}

/// Under an `adapter:` override the completed segment must carry a
/// `MessageStart.v` equal to `brazen::EVENT_SCHEMA_VERSION` (§4.4).
fn check_handshake(expect: bool, completion: &Completion) -> Result<(), Error> {
    if expect && completion.handshake_v() != Some(EVENT_SCHEMA_VERSION) {
        return Err(Error::HandshakeMismatch {
            found: completion.handshake_v(),
            expected: EVENT_SCHEMA_VERSION,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
