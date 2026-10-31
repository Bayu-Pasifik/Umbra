//! Idle "eyes" mascot. Blinks and glances around on a sparse schedule, so the app wakes
//! only when something actually changes and stays at 0% CPU in between.

use eframe::egui::{Painter, Pos2};

use crate::icons;
use crate::spring::Spring;
use crate::theme;

/// Seconds without activity before the clock gives way to the eyes.
pub const IDLE_AFTER: f64 = 10.0;
/// Fade duration between clock and eyes.
const FADE_SECONDS: f32 = 0.4;
const BLINK_SECONDS: f64 = 0.18;
const BLINK_EVERY: f64 = 4.0;
const LOOK_EVERY: f64 = 2.5;

pub struct Mascot {
    /// 0.0 hidden, 1.0 fully visible.
    alpha: f32,
    look_x: Spring,
    look_y: Spring,
    next_look: f64,
    blink_start: Option<f64>,
    next_blink: f64,
    seed: u32,
}

impl Mascot {
    pub fn new() -> Self {
        Self {
            alpha: 0.0,
            look_x: Spring::new(0.0),
            look_y: Spring::new(0.0),
            next_look: LOOK_EVERY,
            blink_start: None,
            next_blink: BLINK_EVERY,
            seed: 0x9E37_79B9,
        }
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// Small deterministic pseudo-random value in -1.0..=1.0.
    fn rand(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.seed >> 8) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }

    /// Advance the mascot. Returns true if it needs another frame right away.
    pub fn update(&mut self, now: f64, dt: f32, idle: bool) -> bool {
        let step = dt / FADE_SECONDS;
        self.alpha = if idle {
            (self.alpha + step).min(1.0)
        } else {
            (self.alpha - step).max(0.0)
        };
        let fading = self.alpha > 0.0 && self.alpha < 1.0;

        if self.alpha <= 0.0 {
            // Reset the schedule so the eyes start calm next time they appear.
            self.blink_start = None;
            self.next_look = now + LOOK_EVERY;
            self.next_blink = now + BLINK_EVERY;
            return fading;
        }

        if now >= self.next_look {
            let (x, y) = (self.rand(), self.rand());
            self.look_x.set_target(x * 4.0);
            self.look_y.set_target(y * 1.5);
            self.next_look = now + LOOK_EVERY + f64::from(self.rand().abs()) * 1.5;
        }
        if self.blink_start.is_none() && now >= self.next_blink {
            self.blink_start = Some(now);
        }
        if let Some(start) = self.blink_start {
            if now - start >= BLINK_SECONDS {
                self.blink_start = None;
                self.next_blink = now + BLINK_EVERY + f64::from(self.rand().abs()) * 2.0;
            }
        }
        self.look_x.update(dt);
        self.look_y.update(dt);

        fading
            || self.blink_start.is_some()
            || !self.look_x.is_at_rest()
            || !self.look_y.is_at_rest()
    }

    /// Seconds until the mascot next needs a frame while it sits still.
    pub fn next_wake(&self, now: f64) -> Option<f64> {
        if self.alpha <= 0.0 {
            return None;
        }
        Some((self.next_look.min(self.next_blink) - now).max(0.0) + 0.01)
    }

    /// Eye height multiplier: 1.0 open, dipping toward 0.1 mid-blink.
    fn openness(&self, now: f64) -> f32 {
        match self.blink_start {
            Some(start) => {
                let t = ((now - start) / BLINK_SECONDS).clamp(0.0, 1.0) as f32;
                let closed = 1.0 - (t * 2.0 - 1.0).abs(); // 0 -> 1 -> 0
                1.0 - 0.9 * closed
            }
            None => 1.0,
        }
    }

    pub fn draw(&self, painter: &Painter, center: Pos2, alpha: f32, now: f64) {
        let a = alpha * self.alpha;
        if a <= 0.0 {
            return;
        }
        let color = theme::TEXT_PRIMARY.gamma_multiply(a);
        icons::eyes(
            painter,
            center,
            self.look_x.value,
            self.look_y.value,
            self.openness(now),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fades_in_when_idle_and_out_on_activity() {
        let mut m = Mascot::new();
        for _ in 0..60 {
            m.update(0.0, 1.0 / 60.0, true);
        }
        assert_eq!(m.alpha(), 1.0);
        for _ in 0..60 {
            m.update(0.0, 1.0 / 60.0, false);
        }
        assert_eq!(m.alpha(), 0.0);
    }

    #[test]
    fn sleeps_between_events() {
        let mut m = Mascot::new();
        for _ in 0..60 {
            m.update(0.0, 1.0 / 60.0, true);
        }
        // Fully visible and still: no immediate repaint needed, and a wake is scheduled.
        assert!(!m.update(0.0, 1.0 / 60.0, true));
        assert!(m.next_wake(0.0).unwrap() > 1.0);
        assert!(Mascot::new().next_wake(0.0).is_none());
    }

    #[test]
    fn blink_closes_and_reopens() {
        let mut m = Mascot::new();
        for _ in 0..60 {
            m.update(0.0, 1.0 / 60.0, true);
        }
        m.blink_start = Some(10.0);
        assert!(m.openness(10.09) < 0.2);
        assert_eq!(m.openness(10.0), 1.0);
    }
}
