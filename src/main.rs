//! Umbra: a Nusa-style dynamic island for the desktop.

#[allow(dead_code)] // state/alpha consumed by widgets in later tasks
mod island;
mod pill;
mod spring;
mod theme;
#[allow(dead_code)] // wired up in task 9
mod widget;

use eframe::egui;

use island::Island;
/// Initial overlay window size: the collapsed pill.
const WINDOW_SIZE: egui::Vec2 = egui::vec2(theme::COLLAPSED_WIDTH, theme::COLLAPSED_HEIGHT);

struct UmbraApp {
    island: Island,
    /// Last window size sent to the OS, to avoid redundant resize commands.
    window_size: egui::Vec2,
    /// Last window x position sent to the OS.
    window_x: Option<f32>,
}

impl UmbraApp {
    fn new() -> Self {
        Self {
            island: Island::new(),
            window_size: WINDOW_SIZE,
            window_x: None,
        }
    }

    /// Make the window hug the pill, pinned to the top-center of the monitor.
    ///
    /// Clicks outside the pill pass through to the apps below because there is no
    /// window there at all. This works the same on every OS.
    fn fit_window_to_pill(&mut self, ctx: &egui::Context) {
        let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) else {
            return;
        };
        let ppp = ctx.pixels_per_point();
        // Round up to whole physical pixels, plus 1px so edges are never clipped.
        let fit = |v: f32| ((v * ppp).ceil() + 1.0) / ppp;
        let size = egui::vec2(fit(self.island.width()), fit(self.island.height()));
        let x = pill::snap((monitor.x - size.x) / 2.0, ppp);

        if size != self.window_size {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            self.window_size = size;
        }
        if self.window_x != Some(x) {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(x, 0.0)));
            self.window_x = Some(x);
        }
    }
}

impl eframe::App for UmbraApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (dt, now) = ctx.input(|i| (i.stable_dt, i.time));
        self.island.update(dt);
        self.fit_window_to_pill(ctx);

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
