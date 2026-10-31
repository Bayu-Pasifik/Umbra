//! File shelf page: drop files in, drag them back out.

use std::path::{Path, PathBuf};

use eframe::egui::{self, pos2, Align2, Rect, Sense, Stroke, StrokeKind, Vec2};

use super::{fade, sans, text, SIZE_MEDIUM, SIZE_SMALL};
use crate::icons;
use crate::theme;
use crate::widget::{Request, Widget};

const COLUMNS: usize = 2;
const MAX_VISIBLE: usize = 4;
const CHIP_HEIGHT: f32 = 34.0;
const CHIP_GAP: f32 = 8.0;
const NAME_CHARS: usize = 20;

pub struct Shelf {
    pub files: Vec<PathBuf>,
    request: Option<Request>,
}

impl Shelf {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            request: None,
        }
    }

    /// Add files, skipping duplicates.
    pub fn add_all(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if !self.files.contains(path) {
                self.files.push(path.clone());
            }
        }
    }
}

/// File name shortened to `max` characters, with an ellipsis in the middle of long names.
pub fn short_name(path: &Path, max: usize) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max {
        return name;
    }
    let keep = max.saturating_sub(1);
    let head = keep.div_ceil(2);
    let tail = keep - head;
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(&chars[chars.len() - tail..]);
    out
}

impl Widget for Shelf {
    fn name(&self) -> &'static str {
        "Shelf"
    }

    fn size(&self) -> Vec2 {
        egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT)
    }

    fn state_key(&self) -> Option<&'static str> {
        Some("shelf")
    }

    fn save_state(&self) -> serde_json::Value {
        serde_json::Value::Array(
            self.files
                .iter()
                .map(|p| serde_json::Value::String(p.to_string_lossy().into_owned()))
                .collect(),
        )
    }

    fn load_state(&mut self, value: &serde_json::Value) {
        self.files = value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect()
            })
            .unwrap_or_default();
    }

    fn accept_files(&mut self, paths: &[PathBuf]) -> bool {
        self.add_all(paths);
        true
    }

    fn take_request(&mut self) -> Option<Request> {
        self.request.take()
    }

    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32) {
        let rect = ui.max_rect();
        let white = fade(theme::TEXT_PRIMARY, alpha);
        let dim = fade(theme::TEXT_SECONDARY, alpha);

        text(
            ui,
            pos2(rect.left(), rect.top() + 12.0),
            Align2::LEFT_CENTER,
            "Shelf",
            sans(SIZE_MEDIUM),
            white,
        );
        let subtitle = match self.files.len() {
            0 => "Drop files here".to_string(),
            1 => "1 file".to_string(),
            n => format!("{n} files"),
        };
        text(
            ui,
            pos2(rect.right(), rect.top() + 12.0),
            Align2::RIGHT_CENTER,
            &subtitle,
            sans(SIZE_SMALL),
            dim,
        );

        let chip_width = (rect.width() - CHIP_GAP * (COLUMNS as f32 - 1.0)) / COLUMNS as f32;
        let top = rect.top() + 34.0;
        let mut remove = None;
        for (i, path) in self.files.iter().enumerate().take(MAX_VISIBLE) {
            let (col, row) = (i % COLUMNS, i / COLUMNS);
            let chip = Rect::from_min_size(
                pos2(
                    rect.left() + col as f32 * (chip_width + CHIP_GAP),
                    top + row as f32 * (CHIP_HEIGHT + CHIP_GAP),
                ),
                egui::vec2(chip_width, CHIP_HEIGHT),
            );
            let response = ui.interact(chip, ui.id().with(("chip", i)), Sense::click_and_drag());
            let fill = if response.hovered() && alpha > 0.5 {
                fade(theme::TRACK, alpha * 1.6)
            } else {
                fade(theme::TRACK, alpha)
            };
            ui.painter().rect_filled(chip, 10.0, fill);
            icons::file(
                ui.painter(),
                pos2(chip.left() + 16.0, chip.center().y),
                16.0,
                dim,
            );
            text(
                ui,
                pos2(chip.left() + 32.0, chip.center().y),
                Align2::LEFT_CENTER,
                &short_name(path, NAME_CHARS),
                sans(SIZE_SMALL),
                white,
            );
            if alpha > 0.5 {
                if response.drag_started() {
                    self.request = Some(Request::DragOut(vec![path.clone()]));
                }
                if response.secondary_clicked() {
                    remove = Some(i);
                }
            }
        }
        if let Some(i) = remove {
            self.files.remove(i);
        }
        if self.files.is_empty() {
            let area = Rect::from_min_max(
                pos2(rect.left(), top),
                pos2(rect.right(), rect.bottom() - 16.0),
            );
            ui.painter().rect_stroke(
                area,
                14.0,
                Stroke::new(icons::STROKE, fade(theme::TRACK, alpha)),
                StrokeKind::Inside,
            );
            icons::file(ui.painter(), area.center(), 22.0, dim);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_names_keep_both_ends() {
        let p = PathBuf::from("/tmp/a_really_long_file_name_for_testing.txt");
        let s = short_name(&p, 20);
        assert_eq!(s.chars().count(), 20);
        assert!(s.starts_with("a_really_"));
        assert!(s.ends_with(".txt"));
        assert_eq!(short_name(Path::new("short.txt"), 20), "short.txt");
    }

    #[test]
    fn duplicates_are_ignored() {
        let mut s = Shelf::new();
        let f = PathBuf::from("a.txt");
        s.add_all(&[f.clone(), f.clone()]);
        s.add_all(&[f]);
        assert_eq!(s.files.len(), 1);
    }
}
