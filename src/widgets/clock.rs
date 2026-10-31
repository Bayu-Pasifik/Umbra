//! Clock page: large time plus the date.

use chrono::{Local, Timelike};
use eframe::egui::{self, Align2, Vec2};

use super::{fade, mono, sans, text, SIZE_LARGE, SIZE_SMALL};
use crate::theme;
use crate::widget::Widget;

pub struct Clock {
    pub use_24h: bool,
}

impl Clock {
    pub fn new() -> Self {
        Self { use_24h: true }
    }

    /// Time string such as `14:05` or `2:05 PM`.
    pub fn time_string(&self) -> String {
        let now = Local::now();
        if self.use_24h {
            format!("{:02}:{:02}", now.hour(), now.minute())
        } else {
            let (pm, h) = now.hour12();
            format!("{}:{:02} {}", h, now.minute(), if pm { "PM" } else { "AM" })
        }
    }
}

impl Widget for Clock {
    fn name(&self) -> &'static str {
        "Clock"
    }

    fn size(&self) -> Vec2 {
        egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT)
    }

    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32) {
        let rect = ui.max_rect();
        text(
            ui,
            rect.center() - egui::vec2(0.0, 8.0),
            Align2::CENTER_CENTER,
            &self.time_string(),
            mono(SIZE_LARGE),
            fade(theme::TEXT_PRIMARY, alpha),
        );
        text(
            ui,
            rect.center() + egui::vec2(0.0, 18.0),
            Align2::CENTER_CENTER,
            &Local::now().format("%A, %d %B").to_string(),
            sans(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );
    }

    /// Wake at the next minute boundary; the clock shows no seconds.
    fn next_wake(&self, _now: f64) -> Option<f64> {
        let secs = Local::now().second() as f64;
        Some(60.0 - secs + 0.05)
    }
}
