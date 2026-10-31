//! `Widget` trait and the page stack that holds the content pages.

use eframe::egui::{self, Vec2};

/// One content page shown inside the expanded island.
pub trait Widget {
    /// Short page name, used for debugging and accessibility.
    fn name(&self) -> &'static str;

    /// Preferred size of the expanded panel for this page.
    fn size(&self) -> Vec2;

    /// Draw into `ui`, which is already clipped to the content area.
    /// `alpha` (0..=1) is the page opacity; widgets must scale their colors by it.
    /// Click handling happens here through `ui.interact`.
    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32);

    /// Handle global input such as keyboard shortcuts. Returns true if consumed.
    fn input(&mut self, _input: &egui::InputState) -> bool {
        false
    }

    /// Advance timers. `now` is the egui clock in seconds.
    fn tick(&mut self, _now: f64) {}

    /// Seconds until this widget needs a redraw on its own (for example a clock tick),
    /// or `None` if it is static. Keeps idle CPU at zero.
    fn next_wake(&self, _now: f64) -> Option<f64> {
        None
    }
}

/// An ordered set of pages with a current selection.
pub struct PageStack {
    pages: Vec<Box<dyn Widget>>,
    current: usize,
}

impl PageStack {
    pub fn new(pages: Vec<Box<dyn Widget>>) -> Self {
        assert!(!pages.is_empty(), "page stack needs at least one page");
        Self { pages, current: 0 }
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    /// Select a page by index; out-of-range indices are ignored.
    pub fn select(&mut self, index: usize) {
        if index < self.pages.len() {
            self.current = index;
        }
    }

    pub fn next(&mut self) {
        self.current = (self.current + 1) % self.pages.len();
    }

    pub fn prev(&mut self) {
        self.current = (self.current + self.pages.len() - 1) % self.pages.len();
    }

    pub fn page(&self, index: usize) -> &dyn Widget {
        self.pages[index].as_ref()
    }

    pub fn current_mut(&mut self) -> &mut dyn Widget {
        self.pages[self.current].as_mut()
    }

    pub fn page_mut(&mut self, index: usize) -> &mut dyn Widget {
        self.pages[index].as_mut()
    }

    /// Tick every page so background timers (Pomodoro, water) keep running.
    pub fn tick_all(&mut self, now: f64) {
        for page in &mut self.pages {
            page.tick(now);
        }
    }

    /// Earliest wake request across all pages.
    pub fn next_wake(&self, now: f64) -> Option<f64> {
        self.pages
            .iter()
            .filter_map(|p| p.next_wake(now))
            .min_by(|a, b| a.total_cmp(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy(&'static str, Option<f64>);

    impl Widget for Dummy {
        fn name(&self) -> &'static str {
            self.0
        }
        fn size(&self) -> Vec2 {
            egui::vec2(420.0, 180.0)
        }
        fn draw(&mut self, _ui: &mut egui::Ui, _alpha: f32) {}
        fn next_wake(&self, _now: f64) -> Option<f64> {
            self.1
        }
    }

    fn stack() -> PageStack {
        PageStack::new(vec![
            Box::new(Dummy("a", None)),
            Box::new(Dummy("b", Some(5.0))),
            Box::new(Dummy("c", Some(2.0))),
        ])
    }

    #[test]
    fn navigation_wraps() {
        let mut s = stack();
        s.prev();
        assert_eq!(s.current_index(), 2);
        s.next();
        assert_eq!(s.current_index(), 0);
        s.select(1);
        assert_eq!(s.current_mut().name(), "b");
        s.select(99);
        assert_eq!(s.current_index(), 1);
    }

    #[test]
    fn earliest_wake_wins() {
        assert_eq!(stack().next_wake(0.0), Some(2.0));
    }
}
