//! OS integration boundary. v1 ships mocks only (PRD assumption): real media session,
//! battery and Bluetooth backends can implement these traits later.

use crate::alert::AlertKind;

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub duration: f64,
}

/// Source of now-playing information and transport control.
pub trait ContentSource {
    fn track(&self) -> &Track;
    /// Playback position in seconds at time `now`.
    fn position(&self, now: f64) -> f64;
    fn is_playing(&self) -> bool;
    fn play_pause(&mut self, now: f64);
    fn skip(&mut self, forward: bool, now: f64);
}

/// Mock playlist that advances in real time.
pub struct MockMedia {
    tracks: Vec<Track>,
    index: usize,
    playing: bool,
    /// Position at `anchor` time.
    base: f64,
    anchor: f64,
}

impl MockMedia {
    pub fn new() -> Self {
        let t = |title: &str, artist: &str, duration: f64| Track {
            title: title.into(),
            artist: artist.into(),
            duration,
        };
        Self {
            tracks: vec![
                t("Midnight City", "M83", 243.0),
                t("Nightcall", "Kavinsky", 258.0),
                t("Resonance", "HOME", 213.0),
            ],
            index: 0,
            playing: false,
            base: 0.0,
            anchor: 0.0,
        }
    }
}

impl ContentSource for MockMedia {
    fn track(&self) -> &Track {
        &self.tracks[self.index]
    }

    fn position(&self, now: f64) -> f64 {
        let pos = if self.playing {
            self.base + (now - self.anchor)
        } else {
            self.base
        };
        pos.clamp(0.0, self.track().duration)
    }

    fn is_playing(&self) -> bool {
        self.playing
    }

    fn play_pause(&mut self, now: f64) {
        self.base = self.position(now);
        self.anchor = now;
        self.playing = !self.playing;
    }

    fn skip(&mut self, forward: bool, now: f64) {
        let n = self.tracks.len();
        self.index = if forward {
            (self.index + 1) % n
        } else {
            (self.index + n - 1) % n
        };
        self.base = 0.0;
        self.anchor = now;
    }
}

/// Source of system events that become transient alerts.
pub trait EventSource {
    /// Events that happened since the last poll.
    fn poll(&mut self, now: f64) -> Vec<AlertKind>;
    /// Seconds until the next possible event, so the app can sleep until then.
    fn next_wake(&self, now: f64) -> Option<f64>;
}

/// Mock event source: cycles through sample alerts on a fixed interval.
/// Enabled with the `UMBRA_DEMO_ALERTS` environment variable; silent otherwise.
pub struct DemoEvents {
    enabled: bool,
    next_at: f64,
    step: usize,
}

const DEMO_INTERVAL: f64 = 8.0;

impl DemoEvents {
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var_os("UMBRA_DEMO_ALERTS").is_some(),
            next_at: 3.0,
            step: 0,
        }
    }
}

impl EventSource for DemoEvents {
    fn poll(&mut self, now: f64) -> Vec<AlertKind> {
        if !self.enabled || now < self.next_at {
            return Vec::new();
        }
        let kinds = [
            AlertKind::Charging { percent: 80 },
            AlertKind::Headphones { connected: true },
            AlertKind::Volume { level: 0.6 },
            AlertKind::Brightness { level: 0.4 },
            AlertKind::CapsLock { on: true },
        ];
        let kind = kinds[self.step % kinds.len()].clone();
        self.step += 1;
        self.next_at = now + DEMO_INTERVAL;
        vec![kind]
    }

    fn next_wake(&self, now: f64) -> Option<f64> {
        self.enabled.then(|| (self.next_at - now).max(0.0) + 0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_advances_only_while_playing() {
        let mut m = MockMedia::new();
        assert_eq!(m.position(10.0), 0.0);
        m.play_pause(10.0);
        assert_eq!(m.position(25.0), 15.0);
        m.play_pause(25.0);
        assert_eq!(m.position(100.0), 15.0);
    }

    #[test]
    fn skip_wraps_and_resets_position() {
        let mut m = MockMedia::new();
        m.skip(false, 0.0);
        assert_eq!(m.track().title, "Resonance");
        m.skip(true, 0.0);
        assert_eq!(m.track().title, "Midnight City");
        assert_eq!(m.position(5.0), 0.0);
    }
}
