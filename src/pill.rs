//! The pill shape: geometry and rendering.

use eframe::egui::{self, Rect};

use crate::theme;

/// Snap a value to the physical pixel grid so edges stay crisp at any DPI scale.
pub fn snap(value: f32, pixels_per_point: f32) -> f32 {
    (value * pixels_per_point).round() / pixels_per_point
}

/// Compute the pill rect, flush with the top edge and centered horizontally in `bounds`.
pub fn pill_rect(bounds: Rect, width: f32, height: f32, pixels_per_point: f32) -> Rect {
    let width = snap(width, pixels_per_point);
    let height = snap(height, pixels_per_point);
    let left = snap(bounds.center().x - width / 2.0, pixels_per_point);
    Rect::from_min_size(egui::pos2(left, bounds.top()), egui::vec2(width, height))
}

/// Paint the pill background.
pub fn paint(ui: &egui::Ui, width: f32, height: f32, radius: f32) -> Rect {
    let ppp = ui.ctx().pixels_per_point();
    let rect = pill_rect(ui.max_rect(), width, height, ppp);
    ui.painter().rect_filled(rect, radius, theme::BACKGROUND);
    rect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_aligns_to_physical_pixels() {
        // At 1.5x, 10.2 pt = 15.3 px -> 15 px = 10.0 pt.
        assert!((snap(10.2, 1.5) - 10.0).abs() < 1e-4);
        assert!((snap(10.0, 1.0) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn pill_is_centered_and_flush_with_top() {
        let bounds = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(420.0, 180.0));
        let r = pill_rect(bounds, 200.0, 32.0, 1.0);
        assert_eq!(r.top(), 0.0);
        assert_eq!(r.left(), 110.0);
        assert_eq!(r.width(), 200.0);
    }
}
