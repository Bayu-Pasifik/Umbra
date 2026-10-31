//! Water reminder page: 1.5 L daily goal.

use chrono::{Local, NaiveDate};
use eframe::egui::{self, pos2, Align2, Rect, Vec2};

use super::{click_area, fade, progress_bar, sans, text, SIZE_LARGE, SIZE_SMALL};
use crate::icons;
use crate::theme;
use crate::widget::Widget;

pub const GOAL_ML: u32 = 1500;
pub const GLASS_ML: u32 = 250;

pub struct Water {
    pub drunk_ml: u32,
    pub day: NaiveDate,
}

impl Water {
    pub fn new() -> Self {
        Self {
            drunk_ml: 0,
            day: Local::now().date_naive(),
        }
    }

    /// Reset the counter when the calendar day changes.
    pub fn roll_day(&mut self, today: NaiveDate) {
        if today != self.day {
            self.day = today;
            self.drunk_ml = 0;
        }
    }

    pub fn add_glass(&mut self) {
        self.drunk_ml = (self.drunk_ml + GLASS_ML).min(GOAL_ML * 2);
    }

    pub fn fraction(&self) -> f32 {
        (self.drunk_ml as f32 / GOAL_ML as f32).min(1.0)
    }
}

impl Widget for Water {
    fn name(&self) -> &'static str {
        "Water"
    }

    fn size(&self) -> Vec2 {
        egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT)
    }

    fn tick(&mut self, _now: f64) {
        self.roll_day(Local::now().date_naive());
    }

    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32) {
        let rect = ui.max_rect();
        let accent = fade(theme::ACCENT, alpha);

        icons::drop(ui.painter(), pos2(rect.left() + 12.0, rect.top() + 16.0), 22.0, accent);
        text(
            ui,
            pos2(rect.left() + 32.0, rect.top() + 16.0),
            Align2::LEFT_CENTER,
            "Water",
            sans(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );
        text(
            ui,
            rect.center() - egui::vec2(0.0, 2.0),
            Align2::CENTER_CENTER,
            &format!("{:.2} / {:.1} L", self.drunk_ml as f32 / 1000.0, GOAL_ML as f32 / 1000.0),
            super::mono(SIZE_LARGE),
            fade(theme::TEXT_PRIMARY, alpha),
        );
        let bar = Rect::from_center_size(
            pos2(rect.center().x, rect.bottom() - 32.0),
            egui::vec2(rect.width() * 0.7, 6.0),
        );
        progress_bar(ui, bar, self.fraction(), alpha);
        text(
            ui,
            pos2(rect.center().x, rect.bottom() - 10.0),
            Align2::CENTER_CENTER,
            "Tap to add a glass",
            sans(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );

        if click_area(ui, rect, "add", alpha) {
            self.add_glass();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glasses_fill_goal_and_day_rolls_over() {
        let mut w = Water::new();
        for _ in 0..6 {
            w.add_glass();
        }
        assert_eq!(w.fraction(), 1.0);
        let tomorrow = w.day.succ_opt().unwrap();
        w.roll_day(tomorrow);
        assert_eq!(w.drunk_ml, 0);
    }
}
