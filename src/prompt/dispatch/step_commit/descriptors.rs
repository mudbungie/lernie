//! Prune the inherited descriptor snapshot down to the forking role's
//! grant, at the dispatch commit (ARCH §3.3, §5.1, §2.3 step 2).
//!
//! `descriptions/**` is snapshotted **whole** into the first config
//! commit — every tool's schema and every skill's frontmatter the install
//! provides (§3.3 *Descriptions-always population*) — because one config
//! commit serves every role. A role's `tools:` grant then selects from it
//! at request-assembly time ([`super::tools`]). Those two facts are one
//! commit apart, and between them sits the agent's own worktree, which
//! inherits the snapshot entire.
//!
//! That gap is a trap the model falls into, twice reproduced (yog
//! bl-55b1): asked why it could not send a message, an agent
//! `bash`-explored its branch, found `descriptions/tools/message.json`
//! and `descriptions/skills/message.md`, concluded the environment
//! supported messaging, and only after many steps landed on the truth —
//! the tool was documented in its tree and absent from its wire array.
//! `providers.yaml`, the grant's actual home, is removed from the tree by
//! the same dispatch commit (§2.2), so nothing on the branch could tell
//! it otherwise.
//!
//! It is also §5.1 violated in the letter: *"Everything inside a branch's
//! worktree is composed into that agent's prompt. There is no exclusion
//! list, no filter."* A non-granted tool's descriptors compose **nowhere**
//! — the body walk skips `descriptions/tools/**` and every tool-claimed
//! skill description (§3.3 *two wire homes*), and the tools array does not
//! carry what the role did not declare. They are worktree bytes with no
//! wire home, reachable only by `bash`.
//!
//! So the fork removes them, at the one place the tree is already trimmed
//! to what the agent may hold. What is left is exact: `descriptions/tools/`
//! **is** the callable set, and "why can't you do X" is answered from the
//! agent's own branch in one `ls`. No second copy of the grant is written,
//! so nothing can drift from it.
//!
//! **Standalone skills stay.** Only a skill some tool claims — one with a
//! `descriptions/tools/<name>.json` beside it — is pruned with its tool.
//! A skill no tool claims composes as a path-framed head text block (§3.3,
//! §5.2) and is `load_skill`-able; it is granted by being present.

use crate::prompt::Error;
use crate::template::GitRunner;
use std::path::Path;

/// Worktree-relative home of the committed tool schemas (§3.3).
const TOOLS_DIR: &str = "descriptions/tools";
/// Worktree-relative home of the committed skill frontmatter (§3.3).
const SKILLS_DIR: &str = "descriptions/skills";

/// Stage the removal of every descriptor `granted` does not cover.
///
/// Issues **no** git command when there is nothing to prune — the shipped
/// default, whose `worker` grant is the whole pool (§4.3), and equally a
/// fork off a parent tip already pruned to the same grant. Idempotent for
/// that reason, and `--ignore-unmatch` keeps a tool whose skill
/// frontmatter never snapshotted from being a failure.
pub(crate) fn prune_ungranted(
    worktree: &Path,
    granted: &[String],
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let mut paths: Vec<String> = Vec::new();
    for name in ungranted(worktree, granted)? {
        paths.push(format!("{TOOLS_DIR}/{name}.json"));
        paths.push(format!("{SKILLS_DIR}/{name}.md"));
    }
    if paths.is_empty() {
        return Ok(());
    }
    let mut args: Vec<&str> = vec!["rm", "-q", "--ignore-unmatch", "--"];
    args.extend(paths.iter().map(String::as_str));
    git.run(worktree, &args).map_err(|source| Error::Git {
        op: "rm ungranted descriptors",
        source,
    })
}

/// The tool names this tree carries a schema for that `granted` does not
/// list, sorted so the staged removal is deterministic.
///
/// A tree with no `descriptions/tools/` at all yields none: nothing was
/// snapshotted, so nothing composes and nothing is stranded. That is the
/// ordinary case for a child forked off a parent tip whose own dispatch
/// already pruned, and for the stub-git unit fixtures.
fn ungranted(worktree: &Path, granted: &[String]) -> Result<Vec<String>, Error> {
    let entries = match std::fs::read_dir(worktree.join(TOOLS_DIR)) {
        Ok(iter) => iter,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().to_string_lossy().into_owned();
            file.strip_suffix(".json").map(str::to_owned)
        })
        .filter(|name| !granted.iter().any(|g| g == name))
        .collect();
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests;
