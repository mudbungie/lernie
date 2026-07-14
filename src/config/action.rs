//! Workflow actions: a small DSL embedded in YAML strings.
//!
//! Examples (from `docs/ARCHITECTURE.md` §6):
//! - `spawn_exchange`
//! - `dispatch(worker)`
//! - `dispatch(worker, with: verifier.feedback)`
//! - `dispatch(compactor, mode: intermediate)`
//! - `gate_return_on(verifier.approve)`
//! - `deliver_result`
//! - `compaction_merge`
//! - `mark_abandoned`
//! - `notify_ui`
//!
//! The closed action set is enumerated by [`Action`]; arity and named-arg
//! validity is enforced when parsing the source string.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One workflow action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    SpawnExchange,
    SpawnRootAgent,
    Dispatch {
        role: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        with: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<DispatchMode>,
    },
    /// Hold a worker's result delivery until the gating predicate holds
    /// (ARCH §6 "gate_return_on — the delivery-hold"): the held state is a
    /// disk query, never a stored flag.
    GateReturnOn {
        predicate: String,
    },
    /// Deliver a (possibly gate-held) result message + work-product
    /// transfer (ARCH §2.6). Lifts a `gate_return_on` hold on approval.
    DeliverResult,
    /// The one merge (ARCH §2.6): land a returning compactor's branch
    /// `--no-ff` at a step boundary. Bound to `compactor_return`.
    CompactionMerge,
    MarkAbandoned,
    NotifyUi,
}

/// Optional `mode:` argument on `dispatch`. Currently `intermediate` is the
/// only named mode; the default (no `mode:`) means a normal terminal
/// dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DispatchMode {
    Intermediate,
}

impl Action {
    /// Parse a workflow action from its YAML-string form.
    pub fn parse(src: &str) -> Result<Self, String> {
        let trimmed = src.trim();
        let (name, args) = split_call(trimmed)?;
        match name {
            "spawn_exchange" => no_args(name, &args).map(|_| Action::SpawnExchange),
            "spawn_root_agent" => no_args(name, &args).map(|_| Action::SpawnRootAgent),
            "deliver_result" => no_args(name, &args).map(|_| Action::DeliverResult),
            "compaction_merge" => no_args(name, &args).map(|_| Action::CompactionMerge),
            "mark_abandoned" => no_args(name, &args).map(|_| Action::MarkAbandoned),
            "notify_ui" => no_args(name, &args).map(|_| Action::NotifyUi),
            "dispatch" => parse_dispatch(&args),
            "gate_return_on" => parse_gate_return_on(&args),
            other => Err(format!("unknown action {other:?}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Arg {
    Positional(String),
    Named { key: String, value: String },
}

fn split_call(src: &str) -> Result<(&str, Vec<Arg>), String> {
    match src.find('(') {
        None => {
            validate_ident(src)?;
            Ok((src, Vec::new()))
        }
        Some(open) => {
            if !src.ends_with(')') {
                return Err(format!("missing closing ')' in {src:?}"));
            }
            let name = &src[..open];
            validate_ident(name)?;
            let inner = &src[open + 1..src.len() - 1];
            let args = parse_arg_list(inner)?;
            Ok((name, args))
        }
    }
}

fn parse_arg_list(inner: &str) -> Result<Vec<Arg>, String> {
    let inner = inner.trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(|raw| parse_arg(raw.trim())).collect()
}

fn parse_arg(raw: &str) -> Result<Arg, String> {
    if raw.is_empty() {
        return Err("empty argument".into());
    }
    if let Some((k, v)) = raw.split_once(':') {
        let key = k.trim();
        let value = v.trim();
        validate_ident(key)?;
        validate_value(value)?;
        Ok(Arg::Named {
            key: key.into(),
            value: value.into(),
        })
    } else {
        validate_value(raw)?;
        Ok(Arg::Positional(raw.into()))
    }
}

fn validate_ident(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("empty identifier".into());
    }
    let ok = s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok {
        return Err(format!("not a valid identifier: {s:?}"));
    }
    Ok(())
}

fn validate_value(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("empty value".into());
    }
    let ok = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    if !ok {
        return Err(format!("not a valid value: {s:?}"));
    }
    Ok(())
}

fn no_args(name: &str, args: &[Arg]) -> Result<(), String> {
    if !args.is_empty() {
        return Err(format!("{name} takes no arguments"));
    }
    Ok(())
}

fn parse_dispatch(args: &[Arg]) -> Result<Action, String> {
    let role = match args.first() {
        Some(Arg::Positional(role)) => role.clone(),
        _ => return Err("dispatch requires a positional role argument".into()),
    };
    let mut with = None;
    let mut mode = None;
    for arg in &args[1..] {
        match arg {
            Arg::Named { key, value } => match key.as_str() {
                "with" => with = Some(value.clone()),
                "mode" => mode = Some(parse_mode(value)?),
                other => return Err(format!("dispatch: unknown named arg {other:?}")),
            },
            Arg::Positional(_) => {
                return Err("dispatch takes at most one positional argument".into());
            }
        }
    }
    Ok(Action::Dispatch { role, with, mode })
}

fn parse_mode(value: &str) -> Result<DispatchMode, String> {
    match value {
        "intermediate" => Ok(DispatchMode::Intermediate),
        other => Err(format!("dispatch: unknown mode {other:?}")),
    }
}

fn parse_gate_return_on(args: &[Arg]) -> Result<Action, String> {
    match args {
        [Arg::Positional(predicate)] => Ok(Action::GateReturnOn {
            predicate: predicate.clone(),
        }),
        _ => Err("gate_return_on takes one positional predicate".into()),
    }
}

// Tests for the action DSL parser live in `tests/action_dsl.rs` so this
// file stays under the 300-line code-file limit.
