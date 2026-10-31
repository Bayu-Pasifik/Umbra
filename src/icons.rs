//! Procedural vector icons drawn with egui shapes. No image files (STYLE.md).

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

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

/// Pair of eyes. `look_*` shift the pupils-in-place look (in points); `open` is 1.0 for
/// fully open and approaches 0.1 mid-blink.
pub fn eyes(p: &Painter, c: Pos2, look_x: f32, look_y: f32, open: f32, color: Color32) {
    let (w, h, gap) = (7.0, 12.0, 15.0);
    for dx in [-gap / 2.0, gap / 2.0] {
        let center = pos2(c.x + dx + look_x, c.y + look_y);
        let r = Rect::from_center_size(center, Vec2::new(w, h * open.clamp(0.1, 1.0)));
        p.rect_filled(r, w / 2.0, color);
    }
}

/// Battery outline with a fill proportional to `level` (0..=1) and a terminal nub.
pub fn battery(p: &Painter, c: Pos2, s: f32, level: f32, color: Color32, fill: Color32) {
    let body = Rect::from_center_size(c, Vec2::new(s, s * 0.5));
    p.rect_stroke(body, 3.0, Stroke::new(STROKE, color), StrokeKind::Inside);
    let nub = Rect::from_center_size(pos2(body.right() + 1.5, c.y), Vec2::new(2.5, s * 0.22));
    p.rect_filled(nub, 1.0, color);
    let inner = body.shrink(3.0);
    let width = inner.width() * level.clamp(0.0, 1.0);
    if width > 0.0 {
        let bar = Rect::from_min_size(inner.min, Vec2::new(width, inner.height()));
        p.rect_filled(bar, 1.5, fill);
    }
}

/// Over-ear headphones: a headband arc and two ear cups.
pub fn headphones(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let r = s * 0.42;
    let center = pos2(c.x, c.y + s * 0.08);
    let mut pts = Vec::new();
    let steps = 16;
    for i in 0..=steps {
        let a = std::f32::consts::PI * (1.0 + i as f32 / steps as f32);
        pts.push(pos2(center.x + r * a.cos(), center.y + r * a.sin()));
    }
    p.add(Shape::line(pts, Stroke::new(STROKE, color)));
    for dx in [-r, r] {
        let cup = Rect::from_center_size(
            pos2(center.x + dx, center.y + s * 0.12),
            Vec2::new(s * 0.2, s * 0.38),
        );
        p.rect_filled(cup, 2.5, color);
    }
}

/// Speaker with sound waves; `level` (0..=1) adds waves.
pub fn volume(p: &Painter, c: Pos2, s: f32, level: f32, color: Color32) {
    let stroke = Stroke::new(STROKE, color);
    let body = vec![
        pos2(c.x - s * 0.5, c.y - s * 0.15),
        pos2(c.x - s * 0.25, c.y - s * 0.15),
        pos2(c.x, c.y - s * 0.4),
        pos2(c.x, c.y + s * 0.4),
        pos2(c.x - s * 0.25, c.y + s * 0.15),
        pos2(c.x - s * 0.5, c.y + s * 0.15),
    ];
    p.add(Shape::convex_polygon(body, color, stroke));
    let waves = if level > 0.66 {
        2
    } else if level > 0.0 {
        1
    } else {
        0
    };
    for i in 0..waves {
        let r = s * (0.25 + 0.22 * i as f32);
        let pts: Vec<Pos2> = (0..=8)
            .map(|k| {
                let a = -0.9 + 1.8 * (k as f32 / 8.0);
                pos2(c.x + s * 0.05 + r * a.cos(), c.y + r * a.sin())
            })
            .collect();
        p.add(Shape::line(pts, stroke));
    }
}

/// Sun: a disc with eight rays.
pub fn sun(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let stroke = Stroke::new(STROKE, color);
    p.circle_stroke(c, s * 0.2, stroke);
    for i in 0..8 {
        let a = std::f32::consts::TAU * i as f32 / 8.0;
        let (sin, cos) = a.sin_cos();
        p.line_segment(
            [
                pos2(c.x + cos * s * 0.32, c.y + sin * s * 0.32),
                pos2(c.x + cos * s * 0.48, c.y + sin * s * 0.48),
            ],
            stroke,
        );
    }
}

/// Caps Lock: an up arrow on a bar; filled when on.
pub fn caps_lock(p: &Painter, c: Pos2, s: f32, on: bool, color: Color32) {
    let stroke = Stroke::new(STROKE, color);
    let arrow = vec![
        pos2(c.x, c.y - s * 0.45),
        pos2(c.x + s * 0.4, c.y + s * 0.05),
        pos2(c.x + s * 0.17, c.y + s * 0.05),
        pos2(c.x + s * 0.17, c.y + s * 0.25),
        pos2(c.x - s * 0.17, c.y + s * 0.25),
        pos2(c.x - s * 0.17, c.y + s * 0.05),
        pos2(c.x - s * 0.4, c.y + s * 0.05),
    ];
    p.add(Shape::Path(eframe::epaint::PathShape {
        points: arrow,
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: stroke.into(),
    }));
    // The bar under the key lights up (thick) when Caps Lock is on.
    let bar_h = if on { 3.5 } else { STROKE };
    let bar = Rect::from_center_size(pos2(c.x, c.y + s * 0.42), Vec2::new(s * 0.34, bar_h));
    p.rect_filled(bar, 1.0, color);
}
