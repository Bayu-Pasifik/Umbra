//! Visual constants from STYLE.md.
#![allow(dead_code)] // palette is consumed incrementally by later tasks

use eframe::egui::Color32;

pub const COLLAPSED_WIDTH: f32 = 200.0;
pub const COLLAPSED_HEIGHT: f32 = 32.0;
pub const EXPANDED_WIDTH: f32 = 420.0;
pub const EXPANDED_HEIGHT: f32 = 180.0;
pub const EXPANDED_RADIUS: f32 = 28.0;

pub const BACKGROUND: Color32 = Color32::BLACK;
pub const TEXT_PRIMARY: Color32 = Color32::WHITE;
/// White at 60% alpha (premultiplied).
pub const TEXT_SECONDARY: Color32 = Color32::from_rgba_premultiplied(153, 153, 153, 153);
pub const ACCENT: Color32 = Color32::from_rgb(0x0A, 0x84, 0xFF);
/// White at 15% alpha (premultiplied).
pub const TRACK: Color32 = Color32::from_rgba_premultiplied(38, 38, 38, 38);
