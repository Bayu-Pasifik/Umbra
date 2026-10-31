//! Fullscreen-app detection, so the island stays out of the way of games and videos.
//!
//! A background thread polls once a second and only wakes the UI when the answer changes,
//! so the app stays at 0% CPU while idle. Detection is implemented for Windows; other
//! platforms report "not fullscreen" and the island stays visible.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use eframe::egui;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Whether another app is currently running fullscreen.
#[cfg(windows)]
fn other_app_is_fullscreen() -> bool {
    // https://learn.microsoft.com/windows/win32/api/shellapi/ne-shellapi-query_user_notification_state
    const QUNS_BUSY: i32 = 2; // fullscreen (non-D3D) application
    const QUNS_RUNNING_D3D_FULL_SCREEN: i32 = 3;
    const QUNS_PRESENTATION_MODE: i32 = 4;

    #[link(name = "shell32")]
    extern "system" {
        fn SHQueryUserNotificationState(state: *mut i32) -> i32;
    }

    let mut state = 0;
    // SAFETY: `state` is a valid out pointer for the duration of the call.
    let hr = unsafe { SHQueryUserNotificationState(&mut state) };
    hr >= 0
        && matches!(
            state,
            QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE
        )
}

#[cfg(not(windows))]
fn other_app_is_fullscreen() -> bool {
    false
}

/// Start the watcher thread. The returned flag is true while another app is fullscreen.
pub fn spawn(ctx: egui::Context) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    let shared = Arc::clone(&flag);
    let spawned = thread::Builder::new()
        .name("umbra-fullscreen".into())
        .spawn(move || loop {
            thread::sleep(POLL_INTERVAL);
            let now = other_app_is_fullscreen();
            if shared.swap(now, Ordering::Relaxed) != now {
                ctx.request_repaint();
            }
        });
    if let Err(err) = spawned {
        eprintln!("umbra: fullscreen watcher unavailable: {err}");
    }
    flag
}
