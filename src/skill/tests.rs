use super::*;

const BASH_SKILL: &str =
    "---\nname: bash\ndescription: Run a shell command.\n---\n\n# bash\n\nbody text\n";

#[test]
fn extracts_body_between_fences() {
    let body = frontmatter_yaml(BASH_SKILL).unwrap();
    assert_eq!(body, "name: bash\ndescription: Run a shell command.\n");
}

#[test]
fn parses_extracted_body_into_typed_fields() {
    let body = frontmatter_yaml(BASH_SKILL).unwrap();
    let fm = parse(body).unwrap();
    assert_eq!(
        fm,
        Frontmatter {
            name: "bash".to_string(),
            description: "Run a shell command.".to_string(),
        }
    );
}

#[test]
fn extra_frontmatter_keys_are_tolerated() {
    let md = "---\nname: x\ndescription: d\nlicense: MIT\n---\nbody\n";
    let fm = parse(frontmatter_yaml(md).unwrap()).unwrap();
    assert_eq!(fm.name, "x");
    assert_eq!(fm.description, "d");
}

#[test]
fn missing_opening_fence_yields_none() {
    assert_eq!(frontmatter_yaml("no frontmatter here\n"), None);
    // Opening `---` not on its own line is not a fence.
    assert_eq!(frontmatter_yaml("---name: x\n---\n"), None);
}

#[test]
fn unclosed_block_yields_none() {
    assert_eq!(frontmatter_yaml("---\nname: x\ndescription: d\n"), None);
}

#[test]
fn empty_block_yields_empty_body() {
    // `---\n---` — opening fence immediately followed by the closing
    // fence (i == 0 branch): an empty YAML body.
    assert_eq!(frontmatter_yaml("---\n---\n"), Some(""));
    // An empty body is not a valid Frontmatter (both fields required).
    assert!(parse("").is_err());
}

#[test]
fn fence_at_end_of_file_without_trailing_newline_closes() {
    assert_eq!(
        frontmatter_yaml("---\nname: a\ndescription: b\n---"),
        Some("name: a\ndescription: b\n")
    );
}

#[test]
fn triple_dash_run_inside_body_is_not_a_closing_fence() {
    // A `----` line (four dashes) is `---` followed by `-`, not newline
    // or EOF, so it does not close; the real fence below does.
    let md = "---\nname: a\ndescription: b\n----\nmore: c\n---\n";
    assert_eq!(
        frontmatter_yaml(md),
        Some("name: a\ndescription: b\n----\nmore: c\n")
    );
}

#[test]
fn missing_required_field_is_a_parse_error() {
    assert!(parse("name: only\n").is_err());
}
