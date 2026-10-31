//! Expand/collapse state machine. Width, height and radius are driven by springs
//! toward the target of the current state, so they always interpolate together.

use crate::spring::Spring;
use crate::theme;

/// Grace period before collapsing after the pointer leaves.
pub const LEAVE_DELAY: f64 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Collapsed,
    Expanding,
    Expanded,
    Collapsing,
}

pub struct Island {
    width: Spring,
    height: Spring,
    radius: Spring,
    want_expanded: bool,
    /// Time (seconds) at which the pointer left while expanded.
    leave_since: Option<f64>,
}

impl Island {
    pub fn new() -> Self {
        Self {
            width: Spring::new(theme::COLLAPSED_WIDTH),
            height: Spring::new(theme::COLLAPSED_HEIGHT),
            radius: Spring::new(theme::COLLAPSED_HEIGHT / 2.0),
            want_expanded: false,
            leave_since: None,
        }
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.want_expanded = expanded;
        let (w, h, r) = if expanded {
            (
                theme::EXPANDED_WIDTH,
                theme::EXPANDED_HEIGHT,
                theme::EXPANDED_RADIUS,
            )
        } else {
            (
                theme::COLLAPSED_WIDTH,
                theme::COLLAPSED_HEIGHT,
                theme::COLLAPSED_HEIGHT / 2.0,
            )
        };
        self.width.set_target(w);
        self.height.set_target(h);
        self.radius.set_target(r);
    }

    /// Feed pointer hover state. Expands immediately; collapses once the pointer has been
    /// away for `LEAVE_DELAY` seconds. Rapid toggling just resets the timer.
    pub fn set_hovered(&mut self, hovered: bool, now: f64) {
        if hovered {
            self.leave_since = None;
            if !self.want_expanded {
                self.set_expanded(true);
            }
        } else if self.want_expanded {
            let since = *self.leave_since.get_or_insert(now);
            if now - since >= LEAVE_DELAY {
                self.leave_since = None;
                self.set_expanded(false);
            }
        }
    }

    /// Seconds until a pending collapse fires, if one is armed.
    pub fn leave_remaining(&self, now: f64) -> Option<f64> {
        self.leave_since.map(|s| (LEAVE_DELAY - (now - s)).max(0.0))
    }

    pub fn wants_expanded(&self) -> bool {
        self.want_expanded
    }

    /// Advance all springs by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        self.width.update(dt);
        self.height.update(dt);
        self.radius.update(dt);
    }

    /// True while any spring is still moving; the app only repaints in this case.
    pub fn is_animating(&self) -> bool {
        !(self.width.is_at_rest() && self.height.is_at_rest() && self.radius.is_at_rest())
    }

    pub fn state(&self) -> State {
        match (self.is_animating(), self.want_expanded) {
            (false, false) => State::Collapsed,
            (false, true) => State::Expanded,
            (true, true) => State::Expanding,
            (true, false) => State::Collapsing,
        }
    }

    pub fn width(&self) -> f32 {
        self.width.value
    }

    pub fn height(&self) -> f32 {
        self.height.value
    }

    /// Corner radius, clamped so the shape never self-intersects during overshoot.
    pub fn radius(&self) -> f32 {
        self.radius
            .value
            .clamp(0.0, self.width.value.min(self.height.value) / 2.0)
    }

    /// Expansion progress, 0.0 (collapsed) to 1.0 (expanded).
    pub fn progress(&self) -> f32 {
        let span = theme::EXPANDED_WIDTH - theme::COLLAPSED_WIDTH;
        ((self.width.value - theme::COLLAPSED_WIDTH) / span).clamp(0.0, 1.0)
    }

    /// Content opacity: fades in after 60% of the expansion (STYLE.md).
    pub fn content_alpha(&self) -> f32 {
        if !self.want_expanded {
            return 0.0; // fades out before the collapse starts
        }
        ((self.progress() - 0.6) / 0.4).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(island: &mut Island) {
        for _ in 0..300 {
            island.update(1.0 / 60.0);
        }
    }

    #[test]
    fn starts_collapsed() {
        let i = Island::new();
        assert_eq!(i.state(), State::Collapsed);
        assert_eq!(i.content_alpha(), 0.0);
    }

    #[test]
    fn expands_then_collapses() {
        let mut i = Island::new();
        i.set_expanded(true);
        assert_eq!(i.state(), State::Expanding);
        settle(&mut i);
        assert_eq!(i.state(), State::Expanded);
        assert_eq!(i.width(), theme::EXPANDED_WIDTH);
        assert!((i.content_alpha() - 1.0).abs() < 1e-3);
        i.set_expanded(false);
        assert_eq!(i.state(), State::Collapsing);
        settle(&mut i);
        assert_eq!(i.state(), State::Collapsed);
    }

    #[test]
    fn radius_never_exceeds_half_height() {
        let mut i = Island::new();
        i.set_expanded(true);
        for _ in 0..120 {
            i.update(1.0 / 60.0);
            assert!(i.radius() <= i.height() / 2.0 + 1e-3);
        }
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    #[test]
    fn hover_expands_and_leave_waits_300ms() {
        let mut i = Island::new();
        i.set_hovered(true, 0.0);
        assert!(i.wants_expanded());
        i.set_hovered(false, 1.0);
        assert!(i.wants_expanded(), "must not collapse immediately");
        i.set_hovered(false, 1.2);
        assert!(i.wants_expanded());
        i.set_hovered(false, 1.31);
        assert!(!i.wants_expanded());
    }

    #[test]
    fn re_entering_cancels_pending_collapse() {
        let mut i = Island::new();
        i.set_hovered(true, 0.0);
        i.set_hovered(false, 1.0);
        i.set_hovered(true, 1.2);
        i.set_hovered(false, 1.25);
        i.set_hovered(false, 1.5);
        assert!(i.wants_expanded(), "timer restarted at 1.25");
        i.set_hovered(false, 1.56);
        assert!(!i.wants_expanded());
    }
}
