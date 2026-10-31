//! Umbra: a Nusa-style dynamic island for the desktop.

mod content;
mod icons;
#[allow(dead_code)] // state/alpha consumed by later tasks
mod island;
mod pill;
mod spring;
mod theme;
mod widget;
mod widgets;

use std::time::Duration;

use eframe::egui;

use island::Island;
use widget::PageStack;
use widgets::{
    clock::Clock, music::Music, pomodoro::Pomodoro, todo::Todo, water::Water, SIZE_MEDIUM,
};

/// Initial overlay window size: the collapsed pill.
const WINDOW_SIZE: egui::Vec2 = egui::vec2(theme::COLLAPSED_WIDTH, theme::COLLAPSED_HEIGHT);

/// Padding between the pill edge and page content.
const CONTENT_PADDING: egui::Vec2 = egui::vec2(24.0, 12.0);

/// Wheel movement (points) needed to switch one page.
const SCROLL_THRESHOLD: f32 = 20.0;

struct UmbraApp {
    island: Island,
    pages: PageStack,
    clock: Clock,
    /// Last window size sent to the OS, to avoid redundant resize commands.
    window_size: egui::Vec2,
    /// Last window x position sent to the OS.
    window_x: Option<f32>,
    scroll_accum: f32,
    last_scroll_nav: f64,
}

impl UmbraApp {
    fn new() -> Self {
        let pages: Vec<Box<dyn widget::Widget>> = vec![
            Box::new(Clock::new()),
            Box::new(Music::new(Box::new(content::MockMedia::new()))),
            Box::new(Pomodoro::new()),
            Box::new(Water::new()),
            Box::new(Todo::new()),
        ];
        Self {
            island: Island::new(),
            pages: PageStack::new(pages),
            clock: Clock::new(),
            window_size: WINDOW_SIZE,
            window_x: None,
            scroll_accum: 0.0,
            last_scroll_nav: f64::NEG_INFINITY,
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

    /// Page dots along the bottom edge; click one to jump to that page.
    fn draw_dots(&mut self, ui: &mut egui::Ui, pill: egui::Rect, alpha: f32) {
        const SPACING: f32 = 14.0;
        let count = self.pages.len();
        let y = pill.bottom() - 8.0;
        let start = pill.center().x - SPACING * (count as f32 - 1.0) / 2.0;
        for i in 0..count {
            let center = egui::pos2(start + SPACING * i as f32, y);
            let active = i == self.pages.current_index();
            let (color, radius) = if active {
                (theme::ACCENT, 3.0)
            } else {
                (theme::TEXT_SECONDARY, 2.5)
            };
            ui.painter()
                .circle_filled(center, radius, widgets::fade(color, alpha));
            let hit = egui::Rect::from_center_size(center, egui::vec2(SPACING, SPACING));
            if widgets::click_area(ui, hit, &format!("dot{i}"), alpha) {
                self.pages.select(i);
            }
        }
    }

    /// Mouse wheel and arrow keys switch pages while the island is open.
    fn handle_navigation(&mut self, ctx: &egui::Context, now: f64) {
        if !self.island.wants_expanded() || ctx.wants_keyboard_input() {
            self.scroll_accum = 0.0;
            return;
        }
        let (left, right, scroll) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                // Treat vertical wheel and horizontal scroll alike.
                i.raw_scroll_delta.y + i.raw_scroll_delta.x,
            )
        });
        if left {
            self.pages.prev();
        }
        if right {
            self.pages.next();
        }
        // Accumulate wheel movement; one page per notch, with a short cooldown
        // so a fast spin or trackpad inertia doesn't skip several pages.
        self.scroll_accum += scroll;
        if self.scroll_accum.abs() >= SCROLL_THRESHOLD && now - self.last_scroll_nav > 0.25 {
            if self.scroll_accum < 0.0 {
                self.pages.next();
            } else {
                self.pages.prev();
            }
            self.last_scroll_nav = now;
            self.scroll_accum = 0.0;
        }
    }

    fn draw_island(&mut self, ui: &mut egui::Ui, now: f64) {
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

        // Collapsed face: the clock, fading out as the pill opens.
        let face_alpha = (1.0 - self.island.progress() * 2.5).clamp(0.0, 1.0);
        if face_alpha > 0.0 {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.clock.time_string(),
                widgets::mono(SIZE_MEDIUM),
                widgets::fade(theme::TEXT_PRIMARY, face_alpha),
            );
        }

        // Expanded content, laid out at full size and clipped to the pill.
        let alpha = self.island.content_alpha();
        if alpha > 0.0 {
            let content = egui::Rect::from_min_size(
                egui::pos2(
                    rect.center().x - theme::EXPANDED_WIDTH / 2.0 + CONTENT_PADDING.x,
                    rect.top() + CONTENT_PADDING.y,
                ),
                egui::vec2(
                    theme::EXPANDED_WIDTH - 2.0 * CONTENT_PADDING.x,
                    theme::EXPANDED_HEIGHT - 2.0 * CONTENT_PADDING.y,
                ),
            );
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content));
            child.set_clip_rect(rect.intersect(ui.clip_rect()));
            self.pages.draw(&mut child, alpha);
            self.draw_dots(ui, rect, alpha);
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
        self.pages.tick_all(now);
        self.handle_navigation(ctx, now);
        let fading = self.pages.update(dt);
        self.fit_window_to_pill(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| self.draw_island(ui, now));

        // Wake up again when the pending collapse is due.
        if let Some(remaining) = self.island.leave_remaining(now) {
            ctx.request_repaint_after(Duration::from_secs_f64(remaining + 0.01));
        }
        // Wake for page timers, only while their page is visible.
        if self.island.wants_expanded() {
            if let Some(wait) = self.pages.current_wake(now) {
                ctx.request_repaint_after(Duration::from_secs_f64(wait));
            }
        }
        // Repaint continuously only while a spring is moving.
        if self.island.is_animating() || fading {
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
