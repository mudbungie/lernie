//! Hyphenated-descent tree ordering for the agent view (§2.3, §7.1).
//!
//! Agent ids encode the full descent from the root (`<a>`, `<a>-<b>`,
//! `<a>-<b>-<c>`, …; §2.3) — hierarchy lives in the name, not the
//! filesystem. This derives the render tree purely from the id set: an
//! agent's parent is the longest *other* agent id that is a
//! hyphen-delimited prefix of it; an agent with no such ancestor present is
//! a root row. A pre-order walk yields each agent paired with its nesting
//! depth, children directly under their parent. Nothing is stored — the
//! tree is a query over the ids (PRINCIPLES "Single source of truth").
//!
//! Robust to the hyphen inside a root id (`20260427T160000Z-pre0`): that id
//! is one node and no other agent is a prefix of it, so it renders at depth
//! 0. A child whose intermediate ancestor ref is absent (e.g. a merged-away
//! compactor) attaches to the nearest *present* ancestor.

use super::Agent;

/// One rendered row: an agent and its nesting depth in the descent tree.
pub struct DescentRow<'a> {
    pub depth: usize,
    pub branch: &'a Agent,
}

/// Order `agents` as a descent tree: roots first (id-sorted), each agent's
/// children immediately beneath it (also id-sorted), depth = nesting level.
pub fn descent_order(agents: &[Agent]) -> Vec<DescentRow<'_>> {
    // Sibling order is by id, deterministically, at every level.
    let mut sorted: Vec<usize> = (0..agents.len()).collect();
    sorted.sort_by(|&a, &b| agents[a].agent_id.cmp(&agents[b].agent_id));

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); agents.len()];
    let mut roots: Vec<usize> = Vec::new();
    for &i in &sorted {
        match nearest_ancestor(agents, i) {
            Some(parent) => children[parent].push(i),
            None => roots.push(i),
        }
    }

    let mut rows = Vec::with_capacity(agents.len());
    for &root in &roots {
        walk(root, 0, agents, &children, &mut rows);
    }
    rows
}

fn walk<'a>(
    i: usize,
    depth: usize,
    agents: &'a [Agent],
    children: &[Vec<usize>],
    rows: &mut Vec<DescentRow<'a>>,
) {
    let branch = &agents[i];
    rows.push(DescentRow { depth, branch });
    for &child in &children[i] {
        walk(child, depth + 1, agents, children, rows);
    }
}

/// Index of the longest other agent id `A` with `id == A + "-" + rest` —
/// the nearest present ancestor. `None` when the agent is a root row.
fn nearest_ancestor(agents: &[Agent], i: usize) -> Option<usize> {
    let id = agents[i].agent_id.as_str();
    let mut best: Option<usize> = None;
    for (j, other) in agents.iter().enumerate() {
        if j == i {
            continue;
        }
        let cand = other.agent_id.as_str();
        // A proper hyphen-delimited prefix: `cand` then a `-` then more.
        if id.len() > cand.len()
            && id.as_bytes()[cand.len()] == b'-'
            && id.starts_with(cand)
            && best.is_none_or(|k| cand.len() > agents[k].agent_id.len())
        {
            best = Some(j);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git_tree::AgentState;

    fn agent(id: &str) -> Agent {
        Agent {
            branch_name: format!("agents/{id}"),
            agent_id: id.to_string(),
            tip_oid: "0".repeat(40),
            tip_short_oid: "00000000".into(),
            tip_timestamp_unix: 0,
            steps: vec![],
            preview: None,
            streaming_text: None,
            tool_calls: vec![],
            state: AgentState::Stopped,
            pending_messages: 0,
            declined_transfer: false,
            budget_exhausted: false,
        }
    }

    /// Collect `(depth, id)` in render order.
    fn order(ids: &[&str]) -> Vec<(usize, String)> {
        let agents: Vec<Agent> = ids.iter().map(|id| agent(id)).collect();
        descent_order(&agents)
            .into_iter()
            .map(|r| (r.depth, r.branch.agent_id.clone()))
            .collect()
    }

    #[test]
    fn empty_set_yields_no_rows() {
        assert!(order(&[]).is_empty());
    }

    #[test]
    fn two_roots_with_internal_hyphens_are_both_depth_zero() {
        // Root ids carry a hyphen (timestamp-suffix), yet neither is a
        // prefix of the other, so both render at depth 0.
        let out = order(&["20260427T160000Z-aaaa", "20260427T160001Z-bbbb"]);
        assert_eq!(
            out,
            vec![
                (0, "20260427T160000Z-aaaa".into()),
                (0, "20260427T160001Z-bbbb".into()),
            ]
        );
    }

    #[test]
    fn child_nests_under_parent() {
        let out = order(&["root-x", "root-x-c1"]);
        assert_eq!(out, vec![(0, "root-x".into()), (1, "root-x-c1".into())]);
    }

    #[test]
    fn multi_level_descent_increments_depth() {
        let out = order(&["a-b", "a-b-c", "a-b-c-d"]);
        assert_eq!(
            out,
            vec![
                (0, "a-b".into()),
                (1, "a-b-c".into()),
                (2, "a-b-c-d".into()),
            ]
        );
    }

    #[test]
    fn siblings_render_id_sorted_under_their_parent() {
        let out = order(&["p-0", "p-0-z", "p-0-a"]);
        assert_eq!(
            out,
            vec![(0, "p-0".into()), (1, "p-0-a".into()), (1, "p-0-z".into()),]
        );
    }

    #[test]
    fn absent_intermediate_ancestor_attaches_to_nearest_present() {
        // `a-b-c` is absent (merged-away compactor); its child attaches to
        // the nearest present ancestor `a-b`.
        let out = order(&["a-b", "a-b-c-d"]);
        assert_eq!(out, vec![(0, "a-b".into()), (1, "a-b-c-d".into())]);
    }

    #[test]
    fn prefix_without_hyphen_boundary_is_not_an_ancestor() {
        // `a-bb` is not a child of `a-b` — the boundary char must be `-`.
        let out = order(&["a-b", "a-bb"]);
        assert_eq!(out, vec![(0, "a-b".into()), (0, "a-bb".into())]);
    }
}
