//! Transient alerts: a priority queue that shows one alert at a time for a fixed hold.

use std::mem::discriminant;

/// How long each alert stays on screen (STYLE.md).
pub const HOLD_SECONDS: f64 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub enum AlertKind {
    Charging { percent: u8 },
    Headphones { connected: bool },
    Volume { level: f32 },
    Brightness { level: f32 },
    CapsLock { on: bool },
}

impl AlertKind {
    /// Higher shows first and may preempt a lower-priority alert.
    pub fn priority(&self) -> u8 {
        match self {
            AlertKind::Charging { .. } => 4,
            AlertKind::Headphones { .. } => 3,
            AlertKind::CapsLock { .. } => 2,
            AlertKind::Volume { .. } | AlertKind::Brightness { .. } => 1,
        }
    }

    fn same_slot(&self, other: &AlertKind) -> bool {
        discriminant(self) == discriminant(other)
    }
}

/// What to show for an alert: a title, an optional subtitle and an optional level (0..=1).
pub struct View {
    pub title: &'static str,
    pub subtitle: String,
    pub level: Option<f32>,
}

impl AlertKind {
    pub fn view(&self) -> View {
        match self {
            AlertKind::Charging { percent } => View {
                title: "Charging",
                subtitle: format!("{percent}%"),
                level: Some(*percent as f32 / 100.0),
            },
            AlertKind::Headphones { connected } => View {
                title: "Headphones",
                subtitle: if *connected {
                    "Connected"
                } else {
                    "Disconnected"
                }
                .into(),
                level: None,
            },
            AlertKind::Volume { level } => View {
                title: "Volume",
                subtitle: format!("{}%", (level * 100.0).round()),
                level: Some(*level),
            },
            AlertKind::Brightness { level } => View {
                title: "Brightness",
                subtitle: format!("{}%", (level * 100.0).round()),
                level: Some(*level),
            },
            AlertKind::CapsLock { on } => View {
                title: "Caps Lock",
                subtitle: if *on { "On" } else { "Off" }.into(),
                level: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct Active {
    kind: AlertKind,
    until: f64,
}

#[derive(Default)]
pub struct AlertQueue {
    pending: Vec<AlertKind>,
    active: Option<Active>,
}

impl AlertQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue an alert. A repeat of the kind on screen (for example a volume change
    /// while the volume bar is visible) updates it in place and restarts the hold.
    pub fn push(&mut self, kind: AlertKind, now: f64) {
        if let Some(active) = &mut self.active {
            if active.kind.same_slot(&kind) {
                active.kind = kind;
                active.until = now + HOLD_SECONDS;
                return;
            }
        }
        if let Some(existing) = self.pending.iter_mut().find(|k| k.same_slot(&kind)) {
            *existing = kind;
        } else {
            self.pending.push(kind);
        }
        self.promote(now);
    }

    /// Advance the queue: expire the active alert and start the next one.
    pub fn update(&mut self, now: f64) {
        if self.active.as_ref().is_some_and(|a| now >= a.until) {
            self.active = None;
        }
        self.promote(now);
    }

    /// The alert to display, if any.
    pub fn current(&self) -> Option<&AlertKind> {
        self.active.as_ref().map(|a| &a.kind)
    }

    /// Seconds until the active alert expires.
    pub fn time_left(&self, now: f64) -> Option<f64> {
        self.active.as_ref().map(|a| (a.until - now).max(0.0))
    }

    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Start the best pending alert if idle, or preempt a lower-priority one.
    fn promote(&mut self, now: f64) {
        let Some(best) = self
            .pending
            .iter()
            .enumerate()
            // max_by_key keeps the last maximum; reverse the index so the earliest wins.
            .max_by_key(|(i, k)| (k.priority(), usize::MAX - i))
            .map(|(i, _)| i)
        else {
            return;
        };
        let preempt = match &self.active {
            None => true,
            Some(a) => self.pending[best].priority() > a.kind.priority(),
        };
        if preempt {
            let kind = self.pending.remove(best);
            if let Some(old) = self.active.take() {
                // Show the interrupted alert again later.
                self.pending.push(old.kind);
            }
            self.active = Some(Active {
                kind,
                until: now + HOLD_SECONDS,
            });
        }
    }
}

/// Draw the alert's icon centered on `c`.
fn draw_icon(painter: &eframe::egui::Painter, kind: &AlertKind, c: eframe::egui::Pos2, alpha: f32) {
    use crate::icons;
    use crate::theme;
    use crate::widgets::fade;

    let white = fade(theme::TEXT_PRIMARY, alpha);
    let accent = fade(theme::ACCENT, alpha);
    match kind {
        AlertKind::Charging { percent } => {
            icons::battery(painter, c, 34.0, *percent as f32 / 100.0, white, accent)
        }
        AlertKind::Headphones { .. } => icons::headphones(painter, c, 34.0, accent),
        AlertKind::Volume { level } => icons::volume(painter, c, 30.0, *level, white),
        AlertKind::Brightness { .. } => icons::sun(painter, c, 30.0, white),
        AlertKind::CapsLock { on } => {
            icons::caps_lock(painter, c, 28.0, *on, if *on { accent } else { white })
        }
    }
}
/// Draw the alert panel: a title, a subtitle, and a level bar where it applies.
pub fn draw(ui: &mut eframe::egui::Ui, kind: &AlertKind, alpha: f32) {
    use crate::theme;
    use crate::widgets::{fade, mono, progress_bar, sans, text, SIZE_LARGE, SIZE_MEDIUM};
    use eframe::egui::{self, pos2, Align2, Rect};

    let rect = ui.max_rect();
    let view = kind.view();
    let has_level = view.level.is_some();
    let y = rect.center().y - if has_level { 14.0 } else { 6.0 };
    draw_icon(
        ui.painter(),
        kind,
        pos2(rect.center().x - 110.0, rect.center().y),
        alpha,
    );
    text(
        ui,
        pos2(rect.center().x, y),
        Align2::CENTER_CENTER,
        view.title,
        sans(SIZE_MEDIUM),
        fade(theme::TEXT_PRIMARY, alpha),
    );
    text(
        ui,
        pos2(rect.center().x, y + 28.0),
        Align2::CENTER_CENTER,
        &view.subtitle,
        mono(SIZE_LARGE),
        fade(theme::TEXT_SECONDARY, alpha),
    );
    if let Some(level) = view.level {
        let bar = Rect::from_center_size(
            pos2(rect.center().x, y + 62.0),
            egui::vec2(rect.width() * 0.6, 6.0),
        );
        progress_bar(ui, bar, level, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vol(level: f32) -> AlertKind {
        AlertKind::Volume { level }
    }

    #[test]
    fn shows_for_two_seconds_then_clears() {
        let mut q = AlertQueue::new();
        q.push(AlertKind::CapsLock { on: true }, 10.0);
        assert!(q.is_active());
        q.update(11.9);
        assert!(q.is_active());
        q.update(12.0);
        assert!(!q.is_active());
    }

    #[test]
    fn same_kind_updates_in_place_and_extends_hold() {
        let mut q = AlertQueue::new();
        q.push(vol(0.2), 0.0);
        q.push(vol(0.8), 1.5);
        assert_eq!(q.current(), Some(&vol(0.8)));
        q.update(3.0);
        assert!(q.is_active(), "hold restarted at 1.5");
        q.update(3.5);
        assert!(!q.is_active());
    }

    #[test]
    fn queued_alerts_run_in_priority_order() {
        let mut q = AlertQueue::new();
        q.push(vol(0.5), 0.0); // starts immediately
        q.push(AlertKind::CapsLock { on: true }, 0.1); // preempts volume
        assert_eq!(q.current(), Some(&AlertKind::CapsLock { on: true }));
        q.update(2.2);
        // The interrupted volume alert comes back after caps lock.
        assert_eq!(q.current(), Some(&vol(0.5)));
        q.update(4.3);
        assert!(!q.is_active());
    }

    #[test]
    fn lower_priority_waits_its_turn() {
        let mut q = AlertQueue::new();
        q.push(AlertKind::Charging { percent: 50 }, 0.0);
        q.push(vol(0.3), 0.5);
        assert!(matches!(q.current(), Some(AlertKind::Charging { .. })));
        q.update(2.0);
        assert_eq!(q.current(), Some(&vol(0.3)));
        assert!((q.time_left(2.0).unwrap() - HOLD_SECONDS).abs() < 1e-9);
    }
}
