//! Umbra: a Nusa-style dynamic island for the desktop.

use eframe::egui;

struct UmbraApp;

impl eframe::App for UmbraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.label("Umbra");
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 180.0]),
        ..Default::default()
    };
    eframe::run_native("Umbra", options, Box::new(|_cc| Ok(Box::new(UmbraApp))))
}
