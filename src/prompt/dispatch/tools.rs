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

use crate::prompt::Error;
use crate::skill;
use brazen::Tool;
use serde_json::Value;
use std::path::Path;

/// Worktree-relative directory holding the committed tool schemas
/// (ARCH §3.3 — `descriptions/tools/<name>.json`, sent verbatim as
/// `input_schema`).
const TOOLS_DESC_DIR: &str = "descriptions/tools";

/// Worktree-relative directory holding the committed skill frontmatter
/// (ARCH §3.3 — `descriptions/skills/<name>.md`; a tool's `description`
/// is its own skill's frontmatter `description`).
const SKILLS_DESC_DIR: &str = "descriptions/skills";

/// Compose the role's `declared` tools against the schemas present under
/// `<worktree>/descriptions/tools/`. Returns one [`Tool`] per declared
/// name whose schema file exists, in declared order (§3.3, §4.3).
pub(super) fn compose(worktree: &Path, declared: &[String]) -> Result<Vec<Tool>, Error> {
    let mut tools = Vec::with_capacity(declared.len());
    for name in declared {
        let path = worktree.join(TOOLS_DESC_DIR).join(format!("{name}.json"));
        let raw = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            // Not present == not available: the intersection drops it.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(Error::ToolSchemaIo {
                    name: name.clone(),
                    path,
                    source,
                });
            }
        };
        let input_schema: Value =
            serde_json::from_slice(&raw).map_err(|source| Error::ToolSchemaJson {
                name: name.clone(),
                path,
                source,
            })?;
        tools.push(Tool::Custom {
            name: name.clone(),
            description: read_description(worktree, name)?,
            input_schema,
            strict: None,
        });
    }
    Ok(tools)
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
