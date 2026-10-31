//! Real "Now Playing" source for Windows, using the System Media Transport Controls.
//!
//! That is the same session the volume flyout shows, so it covers YouTube in a browser,
//! Spotify, Apple Music, VLC and anything else that reports to Windows. A background
//! thread polls once a second and wakes the UI only when something visible changed.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};

use crate::content::{ContentSource, Track};

const POLL: Duration = Duration::from_secs(1);
/// Seconds between the Windows FILETIME epoch (1601) and the Unix epoch (1970).
const FILETIME_UNIX_OFFSET: f64 = 11_644_473_600.0;

#[derive(Clone)]
struct Snapshot {
    track: Option<Track>,
    playing: bool,
    /// Position in seconds at `sampled`.
    position: f64,
    sampled: Instant,
}

enum Command {
    PlayPause,
    Skip(bool),
}

pub struct SystemMedia {
    shared: Arc<Mutex<Snapshot>>,
    commands: Sender<Command>,
}

impl SystemMedia {
    pub fn spawn(ctx: egui::Context) -> Self {
        let shared = Arc::new(Mutex::new(Snapshot {
            track: None,
            playing: false,
            position: 0.0,
            sampled: Instant::now(),
        }));
        let (tx, rx) = mpsc::channel();
        let worker = Arc::clone(&shared);
        let spawned = std::thread::Builder::new()
            .name("umbra-media".into())
            .spawn(move || run(worker, rx, ctx));
        if let Err(err) = spawned {
            eprintln!("umbra: media watcher unavailable: {err}");
        }
        Self {
            shared,
            commands: tx,
        }
    }
}

fn run(shared: Arc<Mutex<Snapshot>>, commands: mpsc::Receiver<Command>, ctx: egui::Context) {
    let manager = match Manager::RequestAsync().and_then(|op| op.get()) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("umbra: no media session manager: {err}");
            return;
        }
    };
    loop {
        match commands.recv_timeout(POLL) {
            Ok(cmd) => {
                if let Ok(session) = manager.GetCurrentSession() {
                    send_command(&session, cmd);
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let fresh = poll(&manager).unwrap_or(None).unwrap_or(Snapshot {
            track: None,
            playing: false,
            position: 0.0,
            sampled: Instant::now(),
        });
        let mut slot = shared.lock().unwrap();
        let same_track = slot.track.as_ref().map(|t| (&t.title, &t.artist))
            == fresh.track.as_ref().map(|t| (&t.title, &t.artist));
        // A seek shows up as a position that disagrees with our own extrapolation.
        let expected = slot.position + slot.sampled.elapsed().as_secs_f64();
        let jumped = slot.playing && (fresh.position - expected).abs() > 2.5;
        let changed = !same_track || slot.playing != fresh.playing || jumped;
        *slot = fresh;
        drop(slot);
        if changed {
            ctx.request_repaint();
        }
    }
}

fn send_command(session: &Session, cmd: Command) {
    let _ = match cmd {
        Command::PlayPause => session.TryTogglePlayPauseAsync(),
        Command::Skip(true) => session.TrySkipNextAsync(),
        Command::Skip(false) => session.TrySkipPreviousAsync(),
    };
}

fn seconds(t: windows::Foundation::TimeSpan) -> f64 {
    t.Duration as f64 / 1e7
}

fn poll(manager: &Manager) -> windows::core::Result<Option<Snapshot>> {
    let Ok(session) = manager.GetCurrentSession() else {
        return Ok(None);
    };
    let props = session.TryGetMediaPropertiesAsync()?.get()?;
    let title = props.Title()?.to_string();
    if title.trim().is_empty() {
        return Ok(None);
    }
    let artist = props.Artist()?.to_string();
    let playing = session.GetPlaybackInfo()?.PlaybackStatus()? == Status::Playing;

    let timeline = session.GetTimelineProperties()?;
    let duration = (seconds(timeline.EndTime()?) - seconds(timeline.StartTime()?)).max(0.0);
    let mut position = seconds(timeline.Position()?);
    if playing {
        // The reported position is as of `LastUpdatedTime`; catch up to now.
        let updated = timeline.LastUpdatedTime()?.UniversalTime as f64 / 1e7;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64())
            + FILETIME_UNIX_OFFSET;
        if updated > 0.0 && now >= updated {
            position += now - updated;
        }
    }
    if duration > 0.0 {
        position = position.clamp(0.0, duration);
    }
    Ok(Some(Snapshot {
        track: Some(Track {
            title,
            artist,
            duration,
        }),
        playing,
        position,
        sampled: Instant::now(),
    }))
}

impl ContentSource for SystemMedia {
    fn track(&self) -> Option<Track> {
        self.shared.lock().unwrap().track.clone()
    }

    fn position(&self, _now: f64) -> f64 {
        let s = self.shared.lock().unwrap();
        let mut pos = s.position;
        if s.playing {
            pos += s.sampled.elapsed().as_secs_f64();
        }
        match &s.track {
            Some(t) if t.duration > 0.0 => pos.clamp(0.0, t.duration),
            _ => pos.max(0.0),
        }
    }

    fn is_playing(&self) -> bool {
        self.shared.lock().unwrap().playing
    }

    fn play_pause(&mut self, _now: f64) {
        let _ = self.commands.send(Command::PlayPause);
    }

    fn skip(&mut self, forward: bool, _now: f64) {
        let _ = self.commands.send(Command::Skip(forward));
    }
}
