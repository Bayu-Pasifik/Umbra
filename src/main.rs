//! Umbra: a Nusa-style dynamic island for the desktop.

#[allow(dead_code)] // state/alpha consumed by widgets in later tasks
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
        let (dt, now) = ctx.input(|i| (i.stable_dt, i.time));
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
                let hovered = ui
                    .input(|i| i.pointer.hover_pos())
                    .is_some_and(|p| rect.contains(p));
                self.island.set_hovered(hovered, now);
            });

        // Wake up again when the pending collapse is due.
        if let Some(remaining) = self.island.leave_remaining(now) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(remaining + 0.01));
        }

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
