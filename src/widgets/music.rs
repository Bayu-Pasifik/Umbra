//! Music page: now-playing title and artist, synced lyrics, progress and transport controls.

use eframe::egui::{self, pos2, Align2, Rect, Vec2};

use super::{
    click_area, fade, format_mmss, mono, progress_bar, sans, text, until_next_second, SIZE_MEDIUM,
    SIZE_SMALL,
};
use crate::content::ContentSource;
use crate::icons;
use crate::lyrics::Lyrics;
use crate::theme;
use crate::widget::Widget;

/// Longest lyric line shown before it is cut with an ellipsis.
const MAX_LYRIC_CHARS: usize = 46;

pub struct Music {
    source: Box<dyn ContentSource>,
    lyrics: Option<Lyrics>,
    now: f64,
}

impl Music {
    pub fn new(source: Box<dyn ContentSource>, lyrics: Option<Lyrics>) -> Self {
        Self {
            source,
            lyrics,
            now: 0.0,
        }
    }
}

/// Shorten `s` to `max` characters, adding an ellipsis when cut.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
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
        let white = fade(theme::TEXT_PRIMARY, alpha);
        let dim = fade(theme::TEXT_SECONDARY, alpha);
        let track = self.source.track();

        let Some(track) = track else {
            text(
                ui,
                rect.center(),
                Align2::CENTER_CENTER,
                "Nothing playing",
                sans(SIZE_MEDIUM),
                dim,
            );
            return;
        };
        let position = self.source.position(now);
        if let Some(lyrics) = &self.lyrics {
            lyrics.request(&track);
        }

        text(
            ui,
            pos2(rect.left(), rect.top() + 12.0),
            Align2::LEFT_CENTER,
            &truncate(&track.title, 38),
            sans(SIZE_MEDIUM),
            white,
        );
        text(
            ui,
            pos2(rect.left(), rect.top() + 32.0),
            Align2::LEFT_CENTER,
            &truncate(&track.artist, 46),
            sans(SIZE_SMALL),
            dim,
        );

        // Synced lyrics: the line being sung, then the one after it.
        if let Some(view) = self.lyrics.as_ref().and_then(|l| l.at(position)) {
            if let Some(current) = view.current {
                text(
                    ui,
                    pos2(rect.center().x, rect.top() + 62.0),
                    Align2::CENTER_CENTER,
                    &truncate(&current, MAX_LYRIC_CHARS),
                    sans(SIZE_MEDIUM),
                    fade(theme::ACCENT, alpha),
                );
            }
            if let Some(next) = view.next {
                text(
                    ui,
                    pos2(rect.center().x, rect.top() + 84.0),
                    Align2::CENTER_CENTER,
                    &truncate(&next, MAX_LYRIC_CHARS),
                    sans(SIZE_SMALL),
                    dim,
                );
            }
        }

        // Progress bar with elapsed / total times.
        let bar_y = rect.top() + 104.0;
        let bar = Rect::from_min_size(pos2(rect.left(), bar_y), egui::vec2(rect.width(), 4.0));
        let fraction = if track.duration > 0.0 {
            (position / track.duration) as f32
        } else {
            0.0
        };
        progress_bar(ui, bar, fraction, alpha);
        text(
            ui,
            pos2(rect.left(), bar_y + 14.0),
            Align2::LEFT_CENTER,
            &format_mmss(position),
            mono(SIZE_SMALL),
            dim,
        );
        if track.duration > 0.0 {
            text(
                ui,
                pos2(rect.right(), bar_y + 14.0),
                Align2::RIGHT_CENTER,
                &format_mmss(track.duration),
                mono(SIZE_SMALL),
                dim,
            );
        }

        // Transport controls.
        let cy = rect.bottom() - 18.0;
        let cx = rect.center().x;
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

    /// While playing, redraw each second for the progress bar and at each lyric change.
    fn next_wake(&self, now: f64) -> Option<f64> {
        if !self.source.is_playing() {
            return None;
        }
        let mut wait = until_next_second(now);
        if let Some(lyrics) = &self.lyrics {
            let position = self.source.position(now);
            if let Some(next_time) = lyrics.at(position).and_then(|v| v.next_time) {
                wait = wait.min((next_time - position).max(0.0) + 0.02);
            }
        }
        Some(wait)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_with_ellipsis() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghij", 5), "abcd…");
    }
}
