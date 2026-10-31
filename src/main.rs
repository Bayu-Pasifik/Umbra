//! Umbra: a Nusa-style dynamic island for the desktop.

mod alert;
mod content;
mod dragout;
mod icons;
#[allow(dead_code)] // state/alpha consumed by later tasks
mod island;
mod mascot;
mod pill;
mod spring;
mod theme;
mod widget;
mod widgets;

use std::time::Duration;

use eframe::egui;

use alert::AlertQueue;
use content::{DemoEvents, EventSource};
use island::Island;
use mascot::Mascot;
use widget::{PageStack, Request, Widget as _};
use widgets::{
    clock::Clock, music::Music, pomodoro::Pomodoro, shelf::Shelf, todo::Todo, water::Water,
    SIZE_MEDIUM,
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
    alerts: AlertQueue,
    events: Box<dyn EventSource>,
    clock: Clock,
    mascot: Mascot,
    /// Last time the user or an alert touched the island.
    last_active: f64,
    /// Files are being dragged over the window.
    file_hover: bool,
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
            Box::new(Shelf::new()),
        ];
        Self {
            island: Island::new(),
            pages: PageStack::new(pages),
            alerts: AlertQueue::new(),
            events: Box::new(DemoEvents::from_env()),
            clock: Clock::new(),
            mascot: Mascot::new(),
            last_active: 0.0,
            file_hover: false,
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
        self.island.set_hovered(hovered || self.file_hover, now);

        // Collapsed face: the clock, fading out as the pill opens.
        let collapsed_alpha = (1.0 - self.island.progress() * 2.5).clamp(0.0, 1.0);
        let face_alpha = collapsed_alpha * (1.0 - self.mascot.alpha());
        self.mascot
            .draw(ui.painter(), rect.center(), collapsed_alpha, now);
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
            if let Some(kind) = self.alerts.current() {
                alert::draw(&mut child, kind, alpha);
            } else {
                self.pages.draw(&mut child, alpha);
            }
            self.draw_dots(ui, rect, alpha);
        }
    }
}

impl eframe::App for UmbraApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let (dt, now) = ctx.input(|i| (i.stable_dt, i.time));
        // Files dragged over the island open the shelf; dropped files land on it.
        let (dropped, hovering) = ctx.input(|i| {
            let dropped: Vec<std::path::PathBuf> = i
                .raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect();
            (dropped, !i.raw.hovered_files.is_empty())
        });
        if hovering && !self.file_hover {
            self.pages.offer_files(&[]);
        }
        self.file_hover = hovering;
        if !dropped.is_empty() {
            self.pages.offer_files(&dropped);
        }
        for event in self.events.poll(now) {
            self.alerts.push(event, now);
        }
        self.alerts.update(now);
        self.island.set_forced(self.alerts.is_active());
        self.island.update(dt);
        let active =
            self.island.wants_expanded() || self.island.is_animating() || self.alerts.is_active();
        if active {
            self.last_active = now;
        }
        let idle = !active && now - self.last_active >= mascot::IDLE_AFTER;
        let mascot_busy = self.mascot.update(now, dt, idle);
        self.pages.tick_all(now);
        self.handle_navigation(ctx, now);
        let fading = self.pages.update(dt);
        self.fit_window_to_pill(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| self.draw_island(ui, now));

        if let Some(Request::DragOut(files)) = self.pages.take_request() {
            dragout::start(frame, files);
        }

        // Wake up again when the pending collapse is due.
        if let Some(remaining) = self.island.leave_remaining(now) {
            ctx.request_repaint_after(Duration::from_secs_f64(remaining + 0.01));
        }
        // Wake when the alert expires or the next system event is due.
        if let Some(left) = self.alerts.time_left(now) {
            ctx.request_repaint_after(Duration::from_secs_f64(left + 0.01));
        }
        if let Some(wait) = self.events.next_wake(now) {
            ctx.request_repaint_after(Duration::from_secs_f64(wait));
        }
        // Wake for page timers, only while their page is visible.
        if self.island.wants_expanded() {
            if let Some(wait) = self.pages.current_wake(now) {
                ctx.request_repaint_after(Duration::from_secs_f64(wait));
            }
        }
        // Repaint continuously only while a spring is moving.
        // Idle transition, mascot schedule, and the collapsed clock face.
        if !active && !idle {
            let left = mascot::IDLE_AFTER - (now - self.last_active);
            ctx.request_repaint_after(Duration::from_secs_f64(left.max(0.0) + 0.01));
        }
        if let Some(wait) = self.mascot.next_wake(now) {
            ctx.request_repaint_after(Duration::from_secs_f64(wait));
        }
        if !self.island.wants_expanded() {
            if let Some(wait) = self.clock.next_wake(now) {
                ctx.request_repaint_after(Duration::from_secs_f64(wait));
            }
        }
        if self.island.is_animating() || fading || mascot_busy {
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
