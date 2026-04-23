//! egui/eframe frontend for the lernie agent harness.
//!
//! This crate is the desktop UI (§3.5 UI contract in ARCHITECTURE.md): a
//! stateless renderer over the conversation repo. It reads the repo's
//! filesystem and issues user actions exclusively as `lernie <subcommand>`
//! invocations per §3.4.
//!
//! For this skeleton milestone, only argument parsing and a placeholder
//! view are wired up; filesystem watching, CLI outbound, and git-tree
//! rendering land in subsequent tasks.

pub mod cli_outbound;
pub mod fs_watcher;
pub mod git_tree;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(version, about = "egui frontend for the lernie agent harness")]
pub struct Args {
    /// Path to the conversation repo to render.
    #[arg(long)]
    pub repo: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub repo: PathBuf,
}

impl AppState {
    pub fn from_args(args: &Args) -> Self {
        Self {
            repo: args.repo.clone(),
        }
    }
}

pub fn render_placeholder(ui: &mut egui::Ui, state: &AppState) {
    ui.vertical_centered(|ui| {
        ui.heading("lernie");
        ui.label("no conversation loaded");
        ui.label(format!("repo: {}", state.repo.display()));
    });
}

pub fn render_app(ui: &mut egui::Ui, state: &AppState, tree: Option<&git_tree::GitTree>) {
    match tree {
        Some(tree) => git_tree::render(ui, tree),
        None => render_placeholder(ui, state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_requires_repo() {
        assert!(Args::try_parse_from(["lernie-ui-egui"]).is_err());
    }

    #[test]
    fn args_parses_repo() {
        let args = Args::try_parse_from(["lernie-ui-egui", "--repo", "/tmp/x"]).unwrap();
        assert_eq!(args.repo, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn app_state_from_args_copies_repo() {
        let args = Args::try_parse_from(["lernie-ui-egui", "--repo", "/tmp/y"]).unwrap();
        let state = AppState::from_args(&args);
        assert_eq!(state.repo, PathBuf::from("/tmp/y"));
    }

    #[test]
    fn placeholder_renders_without_panicking() {
        let ctx = egui::Context::default();
        let state = AppState {
            repo: PathBuf::from("/tmp/z"),
        };
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_placeholder(ui, &state);
            });
        });
    }

    #[test]
    fn render_app_with_tree_uses_tree_view() {
        let ctx = egui::Context::default();
        let state = AppState {
            repo: PathBuf::from("/tmp/z"),
        };
        let tree = git_tree::GitTree::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_app(ui, &state, Some(&tree));
            });
        });
    }

    #[test]
    fn render_app_without_tree_falls_back_to_placeholder() {
        let ctx = egui::Context::default();
        let state = AppState {
            repo: PathBuf::from("/tmp/z"),
        };
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_app(ui, &state, None);
            });
        });
    }
}
