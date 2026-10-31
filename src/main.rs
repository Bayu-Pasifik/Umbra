//! Umbra: a Nusa-style dynamic island for the desktop.

mod island;
mod pill;
mod spring;
mod theme;

use eframe::egui;

use island::Island;

/// Size of the overlay window: large enough for the fully expanded panel.
const WINDOW_SIZE: egui::Vec2 = egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT);

struct UmbraApp {
    positioned: bool,
    island: Island,
}

impl UmbraApp {
    fn new() -> Self {
        Self {
            positioned: false,
            island: Island::new(),
        }
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
        let dt = ctx.input(|i| i.stable_dt);
        self.island.update(dt);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let rect = pill::paint(
                    ui,
                    self.island.width(),
                    self.island.height(),
                    self.island.radius(),
                );
                // Temporary toggle until hover handling lands (task 6).
                let response = ui.interact(rect, ui.id().with("pill"), egui::Sense::click());
                if response.clicked() {
                    self.island.set_expanded(!self.island.wants_expanded());
                }
            });

        // Repaint only while a spring is moving.
        if self.island.is_animating() {
            ctx.request_repaint();
        }
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
