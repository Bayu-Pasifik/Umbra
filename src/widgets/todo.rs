//! To-do list page.

use eframe::egui::{self, pos2, Align2, Key, Rect, Stroke, Vec2};

use super::{fade, sans, text, SIZE_MEDIUM, SIZE_SMALL};
use crate::icons;
use crate::theme;
use crate::widget::Widget;

const MAX_VISIBLE: usize = 3;
const ROW_HEIGHT: f32 = 28.0;

#[derive(Debug, Clone)]
pub struct Item {
    pub title: String,
    pub done: bool,
}

pub struct Todo {
    pub items: Vec<Item>,
    draft: String,
}

impl Todo {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            draft: String::new(),
        }
    }

    pub fn add(&mut self, title: &str) {
        let title = title.trim();
        if !title.is_empty() {
            self.items.push(Item {
                title: title.to_string(),
                done: false,
            });
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.done = !item.done;
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }

    pub fn pending(&self) -> usize {
        self.items.iter().filter(|i| !i.done).count()
    }
}

impl Widget for Todo {
    fn name(&self) -> &'static str {
        "To-do"
    }

    fn size(&self) -> Vec2 {
        egui::vec2(theme::EXPANDED_WIDTH, theme::EXPANDED_HEIGHT)
    }

    fn draw(&mut self, ui: &mut egui::Ui, alpha: f32) {
        let rect = ui.max_rect();
        let white = fade(theme::TEXT_PRIMARY, alpha);
        let dim = fade(theme::TEXT_SECONDARY, alpha);

        text(
            ui,
            pos2(rect.left(), rect.top() + 12.0),
            Align2::LEFT_CENTER,
            "To-do",
            sans(SIZE_MEDIUM),
            white,
        );
        text(
            ui,
            pos2(rect.right(), rect.top() + 12.0),
            Align2::RIGHT_CENTER,
            &format!("{} left", self.pending()),
            sans(SIZE_SMALL),
            dim,
        );

        // Rows: checkbox toggles, clicking the title toggles too, secondary click removes.
        let mut toggle = None;
        let mut remove = None;
        let list_top = rect.top() + 30.0;
        for (i, item) in self.items.iter().enumerate().take(MAX_VISIBLE) {
            let row = Rect::from_min_size(
                pos2(rect.left(), list_top + i as f32 * ROW_HEIGHT),
                egui::vec2(rect.width(), ROW_HEIGHT),
            );
            let box_rect = Rect::from_center_size(
                pos2(row.left() + 9.0, row.center().y),
                egui::vec2(14.0, 14.0),
            );
            let stroke_color = if item.done {
                fade(theme::ACCENT, alpha)
            } else {
                dim
            };
            ui.painter().rect_stroke(
                box_rect,
                4.0,
                Stroke::new(icons::STROKE, stroke_color),
                egui::StrokeKind::Inside,
            );
            if item.done {
                ui.painter()
                    .rect_filled(box_rect.shrink(3.0), 2.0, fade(theme::ACCENT, alpha));
            }
            text(
                ui,
                pos2(row.left() + 26.0, row.center().y),
                Align2::LEFT_CENTER,
                &item.title,
                sans(SIZE_SMALL),
                if item.done { dim } else { white },
            );
            let response = ui.interact(row, ui.id().with(("row", i)), egui::Sense::click());
            if alpha > 0.5 {
                if response.clicked() {
                    toggle = Some(i);
                }
                if response.secondary_clicked() {
                    remove = Some(i);
                }
            }
        }
        if let Some(i) = toggle {
            self.toggle(i);
        }
        if let Some(i) = remove {
            self.remove(i);
        }
        if self.items.is_empty() {
            text(
                ui,
                pos2(rect.left(), list_top + 14.0),
                Align2::LEFT_CENTER,
                "Nothing to do",
                sans(SIZE_SMALL),
                dim,
            );
        }

        // Add field pinned to the bottom.
        let field = Rect::from_min_size(
            pos2(rect.left(), rect.bottom() - 26.0),
            egui::vec2(rect.width(), 24.0),
        );
        let mut field_ui = ui.new_child(egui::UiBuilder::new().max_rect(field));
        let edit = egui::TextEdit::singleline(&mut self.draft)
            .hint_text("Add a task")
            .font(sans(SIZE_SMALL))
            .desired_width(field.width())
            .frame(false)
            .text_color(white);
        let response = field_ui.add_enabled(alpha > 0.5, edit);
        if response.lost_focus() && field_ui.input(|i| i.key_pressed(Key::Enter)) {
            let draft = std::mem::take(&mut self.draft);
            self.add(&draft);
            response.request_focus();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_toggle_remove() {
        let mut t = Todo::new();
        t.add("  write tests ");
        t.add("   ");
        assert_eq!(t.items.len(), 1);
        assert_eq!(t.items[0].title, "write tests");
        assert_eq!(t.pending(), 1);
        t.toggle(0);
        assert_eq!(t.pending(), 0);
        t.remove(0);
        assert!(t.items.is_empty());
    }
}
