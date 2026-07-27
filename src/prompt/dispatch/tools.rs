//! Compose a role's declared tools into the typed canonical request's
//! `tools` array (ARCH §3.3, §4.3, §4.4).
//!
//! §4.3 lets a role declare its enabled tools as `tools: [...]` in the
//! per-repo `providers.yaml`. §3.3 says each tool's JSON schema is
//! committed under `descriptions/tools/<name>.json` in the branch
//! worktree and is **sent verbatim as the `input_schema`** of that
//! tool's entry in the model call's `tools: [...]` array. This module is
//! the consumer: it turns the declared names plus the on-disk schemas
//! into `Vec<brazen::Tool>` so the model is TOLD which tools it may call
//! (previously omitted — the model flew blind about its toolset).
//!
//! **Availability is the intersection.** A declared tool contributes an
//! entry only if its `descriptions/tools/<name>.json` schema is present
//! in the worktree; a name with no schema file is silently dropped (the
//! `descriptions/**` tree is populated by the descriptions-always
//! mechanism, §3.3 — a role may out-declare what a given conversation
//! carries). A schema file that is present but unreadable or not valid
//! JSON is a config fault, surfaced rather than dropped (PRINCIPLES
//! "Decline illegal operations"). Declared order is preserved.
//!
//! The `description` field (§3.3 point 3) is sourced from the tool's
//! `SKILL.md` frontmatter, snapshotted alongside the schema by the
//! descriptions-always producer (`crate::template::descriptions`) into
//! `descriptions/skills/<name>.md`. A tool whose schema is present but
//! whose skill frontmatter is absent composes with `description: None`
//! — the transient producer-ordering state §3.3 sanctions — rather than
//! being dropped; a present-but-malformed frontmatter is a config fault,
//! surfaced rather than dropped (PRINCIPLES "Decline illegal
//! operations").
//!
//! [`compose`] is the single home for the question "what does this
//! request declare?", and it has three answers to add up, not one:
//!
//! 1. **Election** — the intersection above, what the role may offer.
//! 2. **Injection** — a procedure's own toolset, which no config
//!    declares: the compactor's `write_summary` / `mark_for_deletion`
//!    (§2.7), injected for that role alone.
//! 3. **Closure** — a provider validates the request *as a whole*: a
//!    `tool_use` / `tool_result` pair naming a tool the `tools: [...]`
//!    array omits is refused outright, so the array must additionally
//!    cover every tool the history it ships already names. This one
//!    reads the opposite way from election — a name the history
//!    references cannot be dropped, because the exchange has happened.
//!
//! Declaring is not permitting: what a role may *call* is decided at
//! execution ([`super::tool_step`]), and nothing here widens it.

use crate::prompt::Error;
use crate::prompt::compactor;
use crate::skill;
use brazen::{Content, Message, Tool};
use serde_json::{Value, json};
use std::path::Path;

/// Worktree-relative directory holding the committed tool schemas
/// (ARCH §3.3 — `descriptions/tools/<name>.json`, sent verbatim as
/// `input_schema`).
const TOOLS_DESC_DIR: &str = "descriptions/tools";

/// Worktree-relative directory holding the committed skill frontmatter
/// (ARCH §3.3 — `descriptions/skills/<name>.md`; a tool's `description`
/// is its own skill's frontmatter `description`).
const SKILLS_DESC_DIR: &str = "descriptions/skills";

/// Everything `role`'s next model call declares: its `declared` names
/// intersected with the schemas under `<worktree>/descriptions/tools/`
/// (in declared order), then the built-in toolset its procedure injects,
/// then the closure over `history` (§3.3, §4.3, §2.7).
pub(super) fn compose(
    worktree: &Path,
    role: &str,
    declared: &[String],
    history: &[Message],
) -> Result<Vec<Tool>, Error> {
    let mut tools = Vec::with_capacity(declared.len());
    for name in declared {
        // Not present == not available: the intersection drops it.
        if let Some(input_schema) = read_schema(worktree, name)? {
            tools.push(entry(worktree, name, input_schema)?);
        }
    }
    // §2.7/§6 role-aware resolution: the compactor's fixed pair, never a
    // `providers.yaml` list and never riding `descriptions/**`.
    if role == compactor::COMPACTOR_ROLE {
        tools.extend(compactor::builtin_tool_schemas());
    }
    close_over_history(worktree, &mut tools, history)?;
    Ok(tools)
}

/// Append a declaration for every tool `history` names that `tools` does
/// not already carry (ARCH §3.3 — the request's referential integrity).
///
/// The history is not rewritten to fit the declaration; the declaration
/// is widened to fit the history. Transcript entries are immutable
/// (§2.3) and the wire framing is transcript-backed (§3.3), so the
/// alternative — stripping or textualizing a `tool_use` block whose tool
/// this role does not offer — would make the model call disagree with
/// the branch's own record.
///
/// Where the committed schema exists it is used verbatim, exactly as for
/// an elected tool. Where it does not — a name the model invented, whose
/// exchange nonetheless landed in the transcript — a bare
/// `{"type": "object"}` stands in: the entry exists to make the history
/// legible, not to offer the tool.
fn close_over_history(
    worktree: &Path,
    tools: &mut Vec<Tool>,
    history: &[Message],
) -> Result<(), Error> {
    for name in referenced(history) {
        if tools.iter().any(|t| tool_name(t) == name) {
            continue;
        }
        let input_schema =
            read_schema(worktree, &name)?.unwrap_or_else(|| json!({"type":"object"}));
        tools.push(entry(worktree, &name, input_schema)?);
    }
    Ok(())
}

/// Tool names the `tool_use` blocks of `history` reference, in
/// first-appearance order and deduplicated.
fn referenced(history: &[Message]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for block in history.iter().flat_map(|m| m.content.iter()) {
        if let Content::ToolUse { name, .. } = block
            && !names.iter().any(|seen| seen == name)
        {
            names.push(name.clone());
        }
    }
    names
}

/// The declared name of a composed entry, whichever variant carries it.
fn tool_name(tool: &Tool) -> &str {
    match tool {
        Tool::Custom { name, .. } | Tool::Provider { name, .. } => name,
    }
}

/// One `tools: [...]` entry for `name` around an already-read schema.
fn entry(worktree: &Path, name: &str, input_schema: Value) -> Result<Tool, Error> {
    Ok(Tool::Custom {
        name: name.to_string(),
        description: read_description(worktree, name)?,
        input_schema,
        strict: None,
    })
}

/// The committed `descriptions/tools/<name>.json` schema, sent verbatim
/// as `input_schema` (§3.3). `None` when the file is absent — the caller
/// decides what absence means (dropped for an elected tool, stood in for
/// one the history already names). Present-but-unreadable or
/// present-but-malformed is a config fault, surfaced.
fn read_schema(worktree: &Path, name: &str) -> Result<Option<Value>, Error> {
    let path = worktree.join(TOOLS_DESC_DIR).join(format!("{name}.json"));
    let raw = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(Error::ToolSchemaIo {
                name: name.to_string(),
                path,
                source,
            });
        }
    };
    serde_json::from_slice(&raw)
        .map(Some)
        .map_err(|source| Error::ToolSchemaJson {
            name: name.to_string(),
            path,
            source,
        })
}

/// The `description` for tool `name`, from its skill frontmatter at
/// `<worktree>/descriptions/skills/<name>.md` (§3.3 point 3). Absent
/// frontmatter → `None` (the schema-before-description ordering §3.3
/// sanctions); a present-but-malformed frontmatter is surfaced.
fn read_description(worktree: &Path, name: &str) -> Result<Option<String>, Error> {
    let path = worktree.join(SKILLS_DESC_DIR).join(format!("{name}.md"));
    let body = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(Error::SkillFrontmatterIo {
                name: name.to_string(),
                path,
                source,
            });
        }
    };
    skill::parse(&body)
        .map(|fm| Some(fm.description))
        .map_err(|source| Error::SkillFrontmatter {
            name: name.to_string(),
            path,
            source,
        })
}

#[cfg(test)]
mod tests;
