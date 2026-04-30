use clap::Parser;
use lernie_ui_egui::actions::{
    ActionsState, dispatch_new_prompt, dispatch_stop, new_prompt_enabled, stop_enabled,
};
use lernie_ui_egui::cli_outbound::Cli;
use lernie_ui_egui::git_tree::{ConversationBranch, GitTree};
use lernie_ui_egui::{AppState, Args, render_app};

fn main() -> eframe::Result<()> {
    let args = Args::parse();
    let state = AppState::from_args(&args);
    let tree = match GitTree::from_repo(&state.repo) {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!(
                "lernie-ui-egui: could not read git tree at {:?}: {e}",
                state.repo
            );
            None
        }
    };

    eframe::run_native(
        "lernie",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(App {
                state,
                tree,
                actions: ActionsState::default(),
                cli: Cli::resolve(),
            }))
        }),
    )
}

struct App {
    state: AppState,
    tree: Option<GitTree>,
    actions: ActionsState,
    cli: Cli,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let branches: &[ConversationBranch] = self
                .tree
                .as_ref()
                .map(|t| t.in_flight.as_slice())
                .unwrap_or(&[]);
            render_actions(ui, &mut self.actions, branches, &self.cli, &self.state);
            ui.separator();
            render_app(ui, &self.state, self.tree.as_ref());
        });
    }
}

/// Render the action panel: text input + New-prompt button, branch
/// dropdown + Stop button. Lives in `main.rs` (excluded from the 100%
/// coverage floor) because `egui::Response::clicked()` is unreachable
/// in headless tests; the pure derivation and dispatch helpers it
/// invokes are tested in `lernie_ui_egui::actions::tests`.
fn render_actions(
    ui: &mut egui::Ui,
    state: &mut ActionsState,
    branches: &[ConversationBranch],
    cli: &Cli,
    app: &AppState,
) {
    ui.horizontal(|ui| {
        ui.label("prompt:");
        ui.text_edit_singleline(&mut state.new_prompt_input);
        let enabled = new_prompt_enabled(&state.new_prompt_input);
        if ui
            .add_enabled(enabled, egui::Button::new("New prompt"))
            .clicked()
        {
            spawn_detached(dispatch_new_prompt(cli, &app.repo, &state.new_prompt_input));
            state.new_prompt_input.clear();
        }
    });
    ui.horizontal(|ui| {
        let label = state
            .selected_branch
            .clone()
            .unwrap_or_else(|| "(no branch selected)".to_string());
        egui::ComboBox::from_id_salt("stop-branch")
            .selected_text(label)
            .show_ui(ui, |ui| {
                for branch in branches {
                    ui.selectable_value(
                        &mut state.selected_branch,
                        Some(branch.branch_name.clone()),
                        &branch.branch_name,
                    );
                }
            });
        let enabled = stop_enabled(state.selected_branch.as_deref(), branches);
        if ui.add_enabled(enabled, egui::Button::new("Stop")).clicked()
            && let Some(branch) = state.selected_branch.as_deref()
        {
            spawn_detached(dispatch_stop(cli, &app.repo, branch));
        }
    });
}

/// Detach-and-drain: hand the spawned `Stream` to a background thread
/// so it lives independently of the UI. The thread exits when the
/// harness exits naturally, and dropping the `Stream` after that point
/// is a no-op (the child has already been waited on by the iterator's
/// terminal `Exited` chunk). Errors are printed to stderr and dropped;
/// surfacing them in the UI is out of v0.5 scope.
fn spawn_detached(
    spawn_result: Result<
        lernie_ui_egui::cli_outbound::Stream,
        lernie_ui_egui::cli_outbound::CliError,
    >,
) {
    match spawn_result {
        Ok(stream) => {
            std::thread::spawn(move || for _ in stream {});
        }
        Err(e) => eprintln!("lernie-ui-egui: spawn failed: {e}"),
    }
}
