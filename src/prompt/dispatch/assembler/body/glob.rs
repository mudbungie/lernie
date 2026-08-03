//! Minimal §5.2 glob over `/`-separated worktree-relative paths:
//! literal segments, `*` within a segment, `**` spanning any number of
//! segments — the full vocabulary the spec's manifests use; nothing
//! fancier is coined here.

pub(super) fn glob_match(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let segs: Vec<&str> = path.split('/').collect();
    match_segments(&pat, &segs)
}

fn match_segments(pat: &[&str], segs: &[&str]) -> bool {
    match pat.split_first() {
        None => segs.is_empty(),
        Some((&"**", rest)) => (0..=segs.len()).any(|i| match_segments(rest, &segs[i..])),
        Some((p, rest)) => segs.split_first().is_some_and(|(s, tail)| {
            match_one(p.as_bytes(), s.as_bytes()) && match_segments(rest, tail)
        }),
    }
}

/// `*`-wildcard match within one path segment, byte-wise (byte slicing
/// keeps the recursion UTF-8-agnostic).
fn match_one(pat: &[u8], seg: &[u8]) -> bool {
    match pat.iter().position(|&b| b == b'*') {
        None => pat == seg,
        Some(i) => {
            let (pre, rest) = (&pat[..i], &pat[i + 1..]);
            seg.len() >= pre.len()
                && seg[..pre.len()] == *pre
                && (pre.len()..=seg.len()).any(|k| match_one(rest, &seg[k..]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::glob_match;

    #[test]
    fn glob_vocabulary_literals_star_and_doublestar() {
        for (pattern, path, expect) in [
            ("goal.md", "goal.md", true),
            ("goal.md", "soul.md", false),
            ("*.md", "notes.md", true),
            ("*.md", "docs/notes.md", false),
            ("docs/*", "docs/a.md", true),
            ("docs/*", "docs/sub/a.md", false),
            ("summary/**", "summary/001.md", true),
            ("summary/**", "summary/a/b.md", true),
            ("summary/**", "skills/a.md", false),
            ("**/SKILL.md", "skills/x/SKILL.md", true),
            ("**/SKILL.md", "skills/x/notes.md", false),
            ("a*c", "abc", true),
            ("a*c", "ac", true),
            ("a*c", "abd", false),
            ("a*b*c", "aXbYc", true),
            ("abc", "ab", false),
        ] {
            assert_eq!(
                glob_match(pattern, path),
                expect,
                "{pattern} vs {path} should be {expect}"
            );
        }
    }
}
