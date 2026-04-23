use clap::Parser;
use lernie_ui_egui::{AppState, Args, render_placeholder};

fn main() -> eframe::Result<()> {
    let args = Args::parse();
    let state = AppState::from_args(&args);

    eframe::run_native(
        "lernie",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App { state }))),
    )
}

struct App {
    state: AppState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_placeholder(ui, &self.state);
        });
    }
}
