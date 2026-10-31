//! OS integration boundary. v1 ships mocks only (PRD assumption): real media session,
//! battery and Bluetooth backends can implement these traits later.

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
