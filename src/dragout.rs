//! Drag files out of the island into other apps, using the OS drag-and-drop APIs.
//!
//! Supported on Windows and macOS. Linux has no winit-compatible backend (the `drag`
//! crate only works with GTK windows), so drag-out is a documented no-op there.

use std::path::PathBuf;

/// A 1x1 transparent PNG used as the drag preview image.
#[cfg(any(windows, target_os = "macos"))]
const PREVIEW_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// Begin dragging `files` out of `frame`'s window. Blocks until the drop finishes
/// on Windows. Errors are logged, never fatal.
#[cfg(any(windows, target_os = "macos"))]
pub fn start(frame: &eframe::Frame, files: Vec<PathBuf>) {
    let existing: Vec<PathBuf> = files.into_iter().filter(|f| f.exists()).collect();
    if existing.is_empty() {
        return;
    }
    let result = drag::start_drag(
        frame,
        drag::DragItem::Files(existing),
        drag::Image::Raw(PREVIEW_PNG.to_vec()),
        |_result, _position| {},
        drag::Options::default(),
    );
    if let Err(err) = result {
        eprintln!("umbra: drag-out failed: {err}");
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn start(_frame: &eframe::Frame, _files: Vec<PathBuf>) {
    eprintln!("umbra: drag-out is not supported on this platform");
}
