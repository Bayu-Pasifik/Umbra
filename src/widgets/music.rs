//! Music page: title, artist, progress and transport controls.

use eframe::egui::{self, pos2, Align2, Rect, Vec2};

use super::{click_area, fade, format_mmss, progress_bar, sans, text, SIZE_MEDIUM, SIZE_SMALL};
use crate::content::ContentSource;
use crate::icons;
use crate::theme;
use crate::widget::Widget;

pub struct Music {
    source: Box<dyn ContentSource>,
    now: f64,
}

impl Music {
    pub fn new(source: Box<dyn ContentSource>) -> Self {
        Self { source, now: 0.0 }
    }
}

impl Widget for Music {
    fn name(&self) -> &'static str {
        "Music"
    }

    fn size(&self) -> Vec2 {
        egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT)
    }

    fn tick(&mut self, now: f64) {
        self.now = now;
    }

    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32) {
        let rect = ui.max_rect();
        let now = self.now;
        let (title, artist, duration) = {
            let t = self.source.track();
            (t.title.clone(), t.artist.clone(), t.duration)
        };
        let position = self.source.position(now);

        text(
            ui,
            pos2(rect.left(), rect.top() + 14.0),
            Align2::LEFT_CENTER,
            &title,
            sans(SIZE_MEDIUM),
            fade(theme::TEXT_PRIMARY, alpha),
        );
        text(
            ui,
            pos2(rect.left(), rect.top() + 36.0),
            Align2::LEFT_CENTER,
            &artist,
            sans(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );

        // Progress bar with elapsed / total times.
        let bar_y = rect.center().y + 8.0;
        let bar = Rect::from_min_size(pos2(rect.left(), bar_y), egui::vec2(rect.width(), 4.0));
        progress_bar(ui, bar, (position / duration) as f32, alpha);
        text(
            ui,
            pos2(rect.left(), bar_y + 14.0),
            Align2::LEFT_CENTER,
            &format_mmss(position),
            super::mono(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );
        text(
            ui,
            pos2(rect.right(), bar_y + 14.0),
            Align2::RIGHT_CENTER,
            &format_mmss(duration),
            super::mono(SIZE_SMALL),
            fade(theme::TEXT_SECONDARY, alpha),
        );

        // Transport controls.
        let cy = rect.bottom() - 18.0;
        let cx = rect.center().x;
        let white = fade(theme::TEXT_PRIMARY, alpha);
        let button = |dx: f32| Rect::from_center_size(pos2(cx + dx, cy), egui::vec2(36.0, 30.0));

        icons::skip(ui.painter(), pos2(cx - 50.0, cy), 14.0, false, white);
        if self.source.is_playing() {
            icons::pause(ui.painter(), pos2(cx, cy), 16.0, white);
        } else {
            icons::play(ui.painter(), pos2(cx, cy), 16.0, white);
        }
        icons::skip(ui.painter(), pos2(cx + 50.0, cy), 14.0, true, white);

        if click_area(ui, button(-50.0), "prev", alpha) {
            self.source.skip(false, now);
        }
        if click_area(ui, button(0.0), "play", alpha) {
            self.source.play_pause(now);
        }
        if click_area(ui, button(50.0), "next", alpha) {
            self.source.skip(true, now);
        }
    }

    /// Redraw once per second while playing so the progress bar moves.
    fn next_wake(&self, now: f64) -> Option<f64> {
        self.source
            .is_playing()
            .then(|| super::until_next_second(now))
    }
}
