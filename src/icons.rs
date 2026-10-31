//! Procedural vector icons drawn with egui shapes. No image files (STYLE.md).

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

/// Stroke width for all icons.
pub const STROKE: f32 = 1.5;

/// Right-pointing filled triangle centered on `c`, `s` tall.
fn triangle_right(c: Pos2, s: f32) -> Vec<Pos2> {
    let w = s * 0.85;
    vec![
        pos2(c.x - w * 0.4, c.y - s / 2.0),
        pos2(c.x + w * 0.6, c.y),
        pos2(c.x - w * 0.4, c.y + s / 2.0),
    ]
}

pub fn play(p: &Painter, c: Pos2, s: f32, color: Color32) {
    p.add(Shape::convex_polygon(
        triangle_right(c, s),
        color,
        Stroke::new(STROKE, color),
    ));
}

pub fn pause(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let bar_w = s * 0.28;
    for dx in [-s * 0.22, s * 0.22] {
        let r = Rect::from_center_size(pos2(c.x + dx, c.y), Vec2::new(bar_w, s));
        p.rect_filled(r, 1.5, color);
    }
}

/// Skip button: triangle plus a bar. `forward == false` mirrors it.
pub fn skip(p: &Painter, c: Pos2, s: f32, forward: bool, color: Color32) {
    let dir = if forward { 1.0 } else { -1.0 };
    let tri: Vec<Pos2> = triangle_right(pos2(c.x - dir * s * 0.12, c.y), s)
        .into_iter()
        .map(|q| pos2(c.x + (q.x - c.x) * dir, q.y))
        .collect();
    p.add(Shape::convex_polygon(
        tri,
        color,
        Stroke::new(STROKE, color),
    ));
    let bar = Rect::from_center_size(pos2(c.x + dir * s * 0.5, c.y), Vec2::new(s * 0.14, s));
    p.rect_filled(bar, 1.0, color);
}

/// Water drop: pointed top, round bottom.
pub fn drop(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let r = s * 0.34;
    let center = pos2(c.x, c.y + s * 0.16);
    let tip = pos2(c.x, c.y - s / 2.0);
    let mut pts = vec![tip];
    // Sweep the circle from the right tangent point round the bottom to the left one.
    let start = -0.35_f32; // radians above the horizontal on the right side
    let steps = 18;
    let sweep = std::f32::consts::PI + 2.0 * 0.35;
    for i in 0..=steps {
        let a = start + sweep * (i as f32 / steps as f32);
        pts.push(pos2(center.x + r * a.cos(), center.y + r * a.sin()));
    }
    p.add(Shape::convex_polygon(
        pts,
        Color32::TRANSPARENT,
        Stroke::new(STROKE, color),
    ));
}

/// Filled variant of the drop, used for the water goal progress.
pub fn drop_filled(p: &Painter, c: Pos2, s: f32, color: Color32) {
    drop(p, c, s, color);
    let r = s * 0.34;
    p.circle_filled(pos2(c.x, c.y + s * 0.16), r - STROKE * 0.5, color);
}
