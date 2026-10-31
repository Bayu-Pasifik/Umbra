//! Focus timer page: 25:00 Pomodoro.

use eframe::egui::{self, pos2, Align2, Rect, Vec2};

use super::{
    click_area, fade, format_mmss, mono, progress_bar, sans, text, until_next_second, SIZE_LARGE,
    SIZE_SMALL,
};
use crate::theme;
use crate::widget::Widget;

pub const FOCUS_SECONDS: f64 = 25.0 * 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Running,
    Paused,
    Finished,
}

pub struct Pomodoro {
    remaining: f64,
    phase: Phase,
    last_tick: f64,
}

impl Pomodoro {
    pub fn new() -> Self {
        Self {
            remaining: FOCUS_SECONDS,
            phase: Phase::Idle,
            last_tick: 0.0,
        }
    }

    #[cfg(test)]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[cfg(test)]
    pub fn remaining(&self) -> f64 {
        self.remaining
    }

    /// Start, pause or resume; restart if finished.
    pub fn toggle(&mut self, now: f64) {
        match self.phase {
            Phase::Idle | Phase::Paused => {
                self.phase = Phase::Running;
                self.last_tick = now;
            }
            Phase::Running => self.phase = Phase::Paused,
            Phase::Finished => self.reset(),
        }
    }

    pub fn reset(&mut self) {
        self.remaining = FOCUS_SECONDS;
        self.phase = Phase::Idle;
    }
}

impl Widget for Pomodoro {
    fn name(&self) -> &'static str {
        "Focus"
    }

    fn size(&self) -> Vec2 {
        egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT)
    }

    fn tick(&mut self, now: f64) {
        if self.phase == Phase::Running {
            self.remaining -= now - self.last_tick;
            self.last_tick = now;
            if self.remaining <= 0.0 {
                self.remaining = 0.0;
                self.phase = Phase::Finished;
            }
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32) {
        let rect = ui.max_rect();
        let now = ui.input(|i| i.time);
        let subtitle = match self.phase {
            Phase::Idle => "Focus",
            Phase::Running => "Focusing",
            Phase::Paused => "Paused",
            Phase::Finished => "Done",
        };
        text(
            ui,
            pos2(rect.center().x, rect.top() + 14.0),
            Align2::CENTER_CENTER,
            subtitle,
            sans(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );
        text(
            ui,
            rect.center() + egui::vec2(0.0, 2.0),
            Align2::CENTER_CENTER,
            &format_mmss(self.remaining),
            mono(SIZE_LARGE * 1.6),
            fade(theme::TEXT_PRIMARY, alpha),
        );
        let bar = Rect::from_center_size(
            pos2(rect.center().x, rect.bottom() - 28.0),
            egui::vec2(rect.width() * 0.6, 4.0),
        );
        progress_bar(
            ui,
            bar,
            (1.0 - self.remaining / FOCUS_SECONDS) as f32,
            alpha,
        );
        let hint = match self.phase {
            Phase::Running => "Tap to pause",
            Phase::Finished => "Tap to restart",
            _ => "Tap to start",
        };
        text(
            ui,
            pos2(rect.center().x, rect.bottom() - 10.0),
            Align2::CENTER_CENTER,
            hint,
            sans(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );

        if click_area(ui, rect, "toggle", alpha) {
            self.toggle(now);
        }
    }

    fn next_wake(&self, now: f64) -> Option<f64> {
        (self.phase == Phase::Running).then(|| until_next_second(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_and_finishes() {
        let mut p = Pomodoro::new();
        assert_eq!(p.remaining(), FOCUS_SECONDS);
        p.toggle(100.0);
        p.tick(160.0);
        assert!((p.remaining() - (FOCUS_SECONDS - 60.0)).abs() < 1e-9);
        p.tick(100.0 + FOCUS_SECONDS + 5.0);
        assert_eq!(p.phase(), Phase::Finished);
        assert_eq!(p.remaining(), 0.0);
    }

    #[test]
    fn pause_stops_the_clock() {
        let mut p = Pomodoro::new();
        p.toggle(0.0);
        p.tick(10.0);
        p.toggle(10.0);
        p.tick(500.0);
        assert!((p.remaining() - (FOCUS_SECONDS - 10.0)).abs() < 1e-9);
        p.toggle(500.0);
        p.tick(505.0);
        assert!((p.remaining() - (FOCUS_SECONDS - 15.0)).abs() < 1e-9);
    }
}
