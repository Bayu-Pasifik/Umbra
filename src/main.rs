//! Umbra: a Nusa-style dynamic island for the desktop.

mod pill;
mod theme;

use eframe::egui;

/// Size of the overlay window: large enough for the fully expanded panel.
const WINDOW_SIZE: egui::Vec2 = egui::vec2(420.0, 180.0);

struct UmbraApp {
    positioned: bool,
}

impl UmbraApp {
    fn new() -> Self {
        Self { positioned: false }
    }

    /// Pin the window to the top-center of the primary monitor.
    /// The monitor size is only known once the first frame runs.
    fn place_top_center(&mut self, ctx: &egui::Context) {
        if self.positioned {
            return;
        }
        let monitor = ctx.input(|i| i.viewport().monitor_size);
        if let Some(monitor) = monitor {
            let x = (monitor.x - WINDOW_SIZE.x) / 2.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(x, 0.0)));
            self.positioned = true;
        }
    }
}

impl eframe::App for UmbraApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.place_top_center(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                pill::paint(
                    ui,
                    theme::COLLAPSED_WIDTH,
                    theme::COLLAPSED_HEIGHT,
                    theme::COLLAPSED_HEIGHT / 2.0,
                );
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_resizable(false)
            .with_taskbar(false)
            .with_title("Umbra"),
        ..Default::default()
    };
    eframe::run_native(
        "Umbra",
        options,
        Box::new(|_cc| Ok(Box::new(UmbraApp::new()))),
    )
}
