//! Content pages and shared drawing helpers.

pub mod clock;
pub mod music;
pub mod pomodoro;
pub mod todo;
pub mod water;

use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Pos2, Rect, Sense};

use crate::theme;

/// Font sizes from STYLE.md.
pub const SIZE_SMALL: f32 = 13.0;
pub const SIZE_MEDIUM: f32 = 15.0;
pub const SIZE_LARGE: f32 = 22.0;

pub fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

/// Monospace font for the clock and timers.
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Scale a color's opacity by `alpha`.
pub fn fade(color: Color32, alpha: f32) -> Color32 {
    color.gamma_multiply(alpha)
}

pub fn text(ui: &egui::Ui, pos: Pos2, anchor: Align2, s: &str, font: FontId, color: Color32) {
    ui.painter().text(pos, anchor, s, font, color);
}

/// Horizontal progress bar with a track and an accent fill.
pub fn progress_bar(ui: &egui::Ui, rect: Rect, fraction: f32, alpha: f32) {
    let radius = rect.height() / 2.0;
    ui.painter()
        .rect_filled(rect, radius, fade(theme::TRACK, alpha));
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction > 0.0 {
        let fill = Rect::from_min_size(
            rect.min,
            egui::vec2((rect.width() * fraction).max(rect.height()), rect.height()),
        );
        ui.painter()
            .rect_filled(fill, radius, fade(theme::ACCENT, alpha));
    }
}

/// Invisible click target. Returns true when clicked and the page is visible enough.
pub fn click_area(ui: &mut egui::Ui, rect: Rect, id: &str, alpha: f32) -> bool {
    let response = ui.interact(rect, ui.id().with(id), Sense::click());
    if response.hovered() && alpha > 0.5 {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked() && alpha > 0.5
}

/// Format seconds as `MM:SS`.
pub fn format_mmss(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// Seconds until the next whole-second boundary of `t` (plus a hair, so we wake after it).
pub fn until_next_second(t: f64) -> f64 {
    (1.0 - t.fract()).max(0.001) + 0.002
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmss_formats() {
        assert_eq!(format_mmss(1500.0), "25:00");
        assert_eq!(format_mmss(59.6), "01:00");
        assert_eq!(format_mmss(-3.0), "00:00");
    }
}
