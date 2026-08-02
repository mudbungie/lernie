//! The **result envelope**: how a finished tool call is rendered for the
//! model (ARCH §3.3 *Result envelope*).
//!
//! The executor holds three facts about a finished child — its exit
//! code, its stdout and its stderr — and the wire `tool_result` carries
//! one content string plus an `is_error` flag. This module is the one
//! place that turns the former into the latter, so what the model reads
//! is a single derivation from the capture rather than a shape assembled
//! across the executor and the step driver.
//!
//! Why the exit code is *stated* rather than left to `is_error`
//! (bl-ffc5): `is_error` is one bit, so a model reading it cannot tell
//! exit 1 (the command ran and failed) from exit 127 (the command does
//! not exist) from exit 143 (the harness cancelled it, §2.9) — three
//! different next moves. It is also the least reliable field on the
//! wire, since each provider protocol spells it differently; the content
//! is round-tripped verbatim by every one of them. Codex, the harness
//! gpt-5.x models are tuned against, states the code in the content for
//! exactly this reason, and those models are trained to read it there.
//!
//! Why stderr is surfaced on success too: a command that exits 0 while
//! writing to stderr is the ordinary case for compilers, test runners
//! and anything that logs progress — dropping those bytes hid warnings
//! and deprecations from the agent, and left `2>&1` as the only way to
//! see them. Streams stay *labelled* rather than merged because the
//! capture holds them apart (`subprocess.rs` reads two pipes) and
//! merging would discard that: a tool whose stdout is a JSON product
//! (`load_skill`'s `{status, path}`, `cd`'s `{cwd}`) must not have a
//! diagnostic line spliced into it.

/// Fences the stderr block off from the tool's output. On its own line,
/// and only ever emitted with stderr bytes following it.
const STDERR_MARKER: &str = "--- stderr ---\n";

/// Render the model-facing bytes of one finished tool call.
///
/// The shape is: the exit-code line always, the child's stdout verbatim,
/// then — when the child wrote any — a marked stderr block. An empty
/// stream contributes nothing, which is why the marker is conditional:
/// it announces bytes, and there are none to announce. The exit-code
/// line is unconditional because it is the fact the model cannot obtain
/// any other way, and it also means a silent command no longer renders
/// as empty content (a block some providers refuse outright).
pub(super) fn render(exit_code: i32, stdout: &[u8], stderr: &[u8]) -> Vec<u8> {
    let mut out = format!("Exit code: {exit_code}\n").into_bytes();
    out.extend_from_slice(stdout);
    if !stderr.is_empty() {
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }
        out.extend_from_slice(STDERR_MARKER.as_bytes());
        out.extend_from_slice(stderr);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::render;

    fn rendered(exit_code: i32, stdout: &str, stderr: &str) -> String {
        String::from_utf8(render(exit_code, stdout.as_bytes(), stderr.as_bytes()))
            .expect("ASCII fixtures render as UTF-8")
    }

    /// The common case costs exactly one line: the code, then the output.
    #[test]
    fn success_states_the_code_and_carries_stdout() {
        assert_eq!(rendered(0, "hello\n", ""), "Exit code: 0\nhello\n");
    }

    /// The defect bl-ffc5 names: stderr on a zero exit used to be
    /// dropped, so a warning on a successful build was invisible.
    #[test]
    fn stderr_survives_a_zero_exit() {
        assert_eq!(
            rendered(0, "built\n", "warning: deprecated\n"),
            "Exit code: 0\nbuilt\n--- stderr ---\nwarning: deprecated\n"
        );
    }

    /// The other half: exit 1 and exit 127 are distinguishable, where
    /// `is_error` alone made them the same bit.
    #[test]
    fn the_exit_code_distinguishes_failures() {
        assert_eq!(
            rendered(127, "", "sh: nope: not found\n"),
            "Exit code: 127\n--- stderr ---\nsh: nope: not found\n"
        );
        assert_eq!(
            rendered(1, "", "assertion failed\n"),
            "Exit code: 1\n--- stderr ---\nassertion failed\n"
        );
    }

    /// Stdout that does not end in a newline must not run into the
    /// marker — the marker is only meaningful on its own line.
    #[test]
    fn unterminated_stdout_gains_a_separator_before_the_marker() {
        assert_eq!(
            rendered(0, "no trailing newline", "note\n"),
            "Exit code: 0\nno trailing newline\n--- stderr ---\nnote\n"
        );
    }

    /// No stderr, no marker: an empty stream announces nothing.
    #[test]
    fn an_empty_stderr_emits_no_marker() {
        let out = rendered(3, "partial", "");
        assert_eq!(out, "Exit code: 3\npartial");
        assert!(!out.contains("stderr"));
    }

    /// A silent command still renders as content, so no tool call
    /// produces an empty `tool_result` block.
    #[test]
    fn a_silent_command_is_not_empty_content() {
        assert_eq!(rendered(0, "", ""), "Exit code: 0\n");
    }

    /// Bytes are passed through, not transcoded: the executor's raw
    /// capture reaches the model as the tool wrote it.
    #[test]
    fn non_utf8_bytes_pass_through_untouched() {
        let out = render(0, &[0xff, 0xfe], &[0x80]);
        assert_eq!(out, b"Exit code: 0\n\xff\xfe\n--- stderr ---\n\x80");
    }
}
