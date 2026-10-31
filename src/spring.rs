//! Damped spring with fixed-step integration (STYLE.md: stiffness 300, damping 26, mass 1).

pub const STIFFNESS: f32 = 300.0;
pub const DAMPING: f32 = 26.0;
pub const MASS: f32 = 1.0;

/// Fixed physics step in seconds (120 Hz), independent of frame rate.
const STEP: f32 = 1.0 / 120.0;
/// Upper bound on time consumed per update, so a long stall cannot spiral.
const MAX_FRAME: f32 = 0.1;

const REST_DISTANCE: f32 = 0.01;
const REST_VELOCITY: f32 = 0.01;

#[derive(Debug, Clone)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    accumulator: f32,
}

impl Spring {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            accumulator: 0.0,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// True once the spring has settled on its target.
    pub fn is_at_rest(&self) -> bool {
        (self.value - self.target).abs() < REST_DISTANCE && self.velocity.abs() < REST_VELOCITY
    }

    /// Advance by `dt` seconds using fixed sub-steps (semi-implicit Euler).
    pub fn update(&mut self, dt: f32) {
        if self.is_at_rest() {
            self.value = self.target;
            self.velocity = 0.0;
            self.accumulator = 0.0;
            return;
        }
        self.accumulator += dt.clamp(0.0, MAX_FRAME);
        while self.accumulator >= STEP {
            let force = -STIFFNESS * (self.value - self.target) - DAMPING * self.velocity;
            self.velocity += force / MASS * STEP;
            self.value += self.velocity * STEP;
            self.accumulator -= STEP;
        }
        if self.is_at_rest() {
            self.value = self.target;
            self.velocity = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(spring: &mut Spring, seconds: f32) -> f32 {
        let mut peak = spring.value;
        let frames = (seconds * 60.0) as usize;
        for _ in 0..frames {
            spring.update(1.0 / 60.0);
            peak = peak.max(spring.value);
        }
        peak
    }

    #[test]
    fn settles_on_target() {
        let mut s = Spring::new(0.0);
        s.set_target(100.0);
        run(&mut s, 3.0);
        assert!(s.is_at_rest());
        assert_eq!(s.value, 100.0);
    }

    #[test]
    fn overshoot_is_slight() {
        let mut s = Spring::new(0.0);
        s.set_target(100.0);
        let peak = run(&mut s, 3.0);
        let overshoot = (peak - 100.0) / 100.0;
        assert!(overshoot > 0.0 && overshoot < 0.05, "overshoot {overshoot}");
    }

    #[test]
    fn frame_rate_independent() {
        let mut a = Spring::new(0.0);
        let mut b = Spring::new(0.0);
        a.set_target(50.0);
        b.set_target(50.0);
        for _ in 0..30 {
            a.update(1.0 / 60.0);
        }
        for _ in 0..60 {
            b.update(1.0 / 120.0);
        }
        assert!((a.value - b.value).abs() < 0.5);
    }
}
