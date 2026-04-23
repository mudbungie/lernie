use clap::Parser;
use lernie_ui_egui::{AppState, Args, git_tree::GitTree, render_app};

fn main() -> eframe::Result<()> {
    let args = Args::parse();
    let state = AppState::from_args(&args);
    let tree = match GitTree::from_repo(&state.repo) {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("lernie-ui-egui: could not read git tree at {:?}: {e}", state.repo);
            None
        }
    };

    eframe::run_native(
        "lernie",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App { state, tree }))),
    )
}

struct App {
    state: AppState,
    tree: Option<GitTree>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_app(ui, &self.state, self.tree.as_ref());
        });
    }
}
