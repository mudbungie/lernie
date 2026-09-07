//! **What a wall's policy is written in**: its roles, its config lineages,
//! one config file read against its schema, and the machines registered to
//! execute for it.
//!
//! Split from [`super::records`] at the design-time budget on the seam the two
//! already have: a record is one CONVERSATION's own ledger, and this is the
//! WALL's standing arrangement. The first changes every step; the second only
//! when somebody edits a file.

use crate::reply::clients::ClientRow;
use crate::reply::config::Config;
use crate::reply::lineages::Lineage;
use crate::reply::proposals::Proposals;
use crate::reply::roles::RoleRow;

use super::parts::{clause, line, line_over, listing, quoted, things, when};

/// **What a wall's roles are set to.**
pub(super) fn roles(rows: &[RoleRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.role.clone()),
                Some(row.runs_on()),
                when(row.priority, "priority"),
                clause("effort", row.effort.as_deref()),
            ])
        })
        .collect();
    listing("roles", painted, "this wall sets no role")
}

/// **One config file's bytes, and the settings its schema found in them.**
pub(super) fn config(config: &Config) -> String {
    let settings = config
        .settings
        .iter()
        .map(|setting| {
            line(vec![
                Some(format!("{}.{}", setting.entry, setting.name)),
                Some(format!("= {}", setting.value)),
                Some(setting.control.kind.clone()),
                clause("fault:", setting.fault.as_deref()),
                Some(setting.help.clone()),
            ])
        })
        .collect();
    line_over(
        &listing("settings", settings, "this schema found no setting"),
        Some(config.text.clone()),
    )
}

/// **The config lineages one wall holds.**
pub(super) fn lineages(rows: &[Lineage]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.name.clone()),
                Some(row.short_oid.clone()),
                things(row.files.len() as u64, "file"),
            ])
        })
        .collect();
    listing("lineages", painted, "this wall holds no lineage")
}

/// **What a reviewer has staged for this wall's config** (REMOTE §9.22), and —
/// where the read named one — that proposal whole under the listing.
///
/// The standing is the engine's word and not this seat's arithmetic: `stale`
/// means somebody advanced the lineage after the reviewer read it, so the
/// patch was written against a config that no longer governs anything and the
/// answer is to reject it. The whole is the reviewer's message and diff,
/// verbatim, because a diff a seat re-formatted is a diff nobody can apply.
pub(super) fn proposals(staged: &Proposals) -> String {
    let painted = staged
        .rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.id.clone()),
                Some(row.standing()),
                Some(row.moves()),
                Some(row.diffstat.clone()),
                quoted(&row.subject),
            ])
        })
        .collect();
    line_over(
        &listing("proposals", painted, "nothing is staged for this wall"),
        staged.whole.clone(),
    )
}

/// **The machines registered in one workspace**, and what each offers.
pub(super) fn clients(rows: &[ClientRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line_over(
                &line(vec![
                    Some(row.client.clone()),
                    when(row.present, "present"),
                    // **The absence is the fact** (`clients::NEVER`): on a
                    // command line every row reads "not present", because
                    // every verb opens and closes its own connection — so the
                    // reading a roster is scanned for here is which row has
                    // never dialled at all.
                    when(row.last_seen.is_none(), crate::reply::clients::NEVER),
                    things(row.tools.len() as u64, "tool"),
                ]),
                Some(
                    row.tools
                        .iter()
                        .map(|tool| {
                            line(vec![
                                Some(tool.name.clone()),
                                when(tool.subject_cwd, "(cwd)"),
                                quoted(&tool.description),
                            ])
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                ),
            )
        })
        .collect();
    listing("clients", painted, "no machine is registered here")
}
