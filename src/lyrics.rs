//! Synced lyrics from LRCLIB (https://lrclib.net). This is the only network access in the
//! app and it is optional: set `UMBRA_NO_LYRICS=1` to disable it. Only the track title,
//! artist and duration are sent.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::egui;

use crate::content::Track;

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub time: f64,
    pub text: String,
}

/// Parse LRC text (`[mm:ss.xx] words`). Lines may carry several timestamps; metadata tags
/// such as `[ar:Artist]` are ignored. The result is sorted by time.
pub fn parse_lrc(text: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut rest = raw.trim();
        let mut times = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some(end) = stripped.find(']') else { break };
            match parse_timestamp(&stripped[..end]) {
                Some(t) => times.push(t),
                None => break,
            }
            rest = stripped[end + 1..].trim_start();
        }
        for time in times {
            lines.push(Line {
                time,
                text: rest.trim().to_string(),
            });
        }
    }
    lines.sort_by(|a, b| a.time.total_cmp(&b.time));
    lines
}

fn parse_timestamp(s: &str) -> Option<f64> {
    let (min, sec) = s.split_once(':')?;
    let min: f64 = min.trim().parse().ok()?;
    let sec: f64 = sec.trim().parse().ok()?;
    Some(min * 60.0 + sec)
}

/// Index of the line being sung at `position`, if the song has started its lyrics.
pub fn current_index(lines: &[Line], position: f64) -> Option<usize> {
    lines.iter().rposition(|l| l.time <= position)
}

/// Strip decoration that streaming sites add: "(Official Video)", "[HD]", and so on.
pub fn clean_title(title: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in title.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Best-guess (title, artist) for lookups. YouTube titles are often "Artist - Song" while
/// the reported artist is just the channel name.
pub fn guess_names(track: &Track) -> (String, String) {
    let title = clean_title(&track.title);
    if let Some((artist, song)) = title.split_once(" - ") {
        return (song.trim().to_string(), artist.trim().to_string());
    }
    let artist = track
        .artist
        .trim_end_matches(" - Topic")
        .trim_end_matches("VEVO")
        .trim()
        .to_string();
    (title, artist)
}

fn fetch(track: &Track) -> Option<Vec<Line>> {
    let (title, artist) = guess_names(track);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent("Umbra/0.1 (https://github.com/Bayu-Pasifik/Umbra)")
        .build();

    // Exact match first; it needs a known duration.
    if track.duration > 0.0 {
        let duration = track.duration.round().to_string();
        let exact = agent
            .get("https://lrclib.net/api/get")
            .query("track_name", &title)
            .query("artist_name", &artist)
            .query("duration", &duration)
            .call()
            .ok()
            .and_then(|r| r.into_json::<serde_json::Value>().ok())
            .and_then(|v| synced(&v));
        if exact.is_some() {
            return exact;
        }
    }

    let results: Vec<serde_json::Value> = agent
        .get("https://lrclib.net/api/search")
        .query("q", &format!("{title} {artist}"))
        .call()
        .ok()?
        .into_json()
        .ok()?;
    results
        .iter()
        .filter(|r| {
            let d = r.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
            track.duration <= 0.0 || (d - track.duration).abs() <= 6.0
        })
        .find_map(synced)
}

fn synced(v: &serde_json::Value) -> Option<Vec<Line>> {
    let text = v.get("syncedLyrics")?.as_str()?;
    let lines = parse_lrc(text);
    (!lines.is_empty()).then_some(lines)
}

#[derive(Default)]
struct Slot {
    key: String,
    lines: Option<Vec<Line>>,
}

/// Lyrics for the current track, fetched in the background.
pub struct Lyrics {
    slot: Arc<Mutex<Slot>>,
    ctx: egui::Context,
    enabled: bool,
}

pub struct LyricView {
    pub current: Option<String>,
    pub next: Option<String>,
    pub next_time: Option<f64>,
}

impl Lyrics {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            slot: Arc::new(Mutex::new(Slot::default())),
            ctx,
            enabled: std::env::var_os("UMBRA_NO_LYRICS").is_none(),
        }
    }

    /// Start fetching lyrics for `track` unless that is already done or under way.
    pub fn request(&self, track: &Track) {
        if !self.enabled {
            return;
        }
        let key = format!("{}\u{1}{}", track.title, track.artist);
        {
            let mut slot = self.slot.lock().unwrap();
            if slot.key == key {
                return;
            }
            slot.key = key.clone();
            slot.lines = None;
        }
        let slot = Arc::clone(&self.slot);
        let ctx = self.ctx.clone();
        let track = track.clone();
        let spawned = std::thread::Builder::new()
            .name("umbra-lyrics".into())
            .spawn(move || {
                let lines = fetch(&track);
                let mut slot = slot.lock().unwrap();
                // Ignore the answer if the song changed while we were waiting.
                if slot.key == key {
                    slot.lines = lines;
                    drop(slot);
                    ctx.request_repaint();
                }
            });
        if let Err(err) = spawned {
            eprintln!("umbra: lyrics thread failed: {err}");
        }
    }

    /// Current and next lyric line at `position`, plus the time the next line starts.
    pub fn at(&self, position: f64) -> Option<LyricView> {
        let slot = self.slot.lock().unwrap();
        let lines = slot.lines.as_ref()?;
        let idx = current_index(lines, position);
        let next = idx.map_or(0, |i| i + 1);
        Some(LyricView {
            current: idx.map(|i| lines[i].text.clone()).filter(|t| !t.is_empty()),
            next: lines.get(next).map(|l| l.text.clone()),
            next_time: lines.get(next).map(|l| l.time),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lrc_with_tags_and_multiple_stamps() {
        let lrc = "[ar:Someone]\n[00:12.50] second\n[00:01.00][00:30.00]chorus\n\n[bad] x";
        let lines = parse_lrc(lrc);
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[0],
            Line {
                time: 1.0,
                text: "chorus".into()
            }
        );
        assert_eq!(lines[1].time, 12.5);
        assert_eq!(lines[2].time, 30.0);
    }

    #[test]
    fn finds_current_line() {
        let lines = parse_lrc("[00:05.00]a\n[00:10.00]b");
        assert_eq!(current_index(&lines, 2.0), None);
        assert_eq!(current_index(&lines, 5.0), Some(0));
        assert_eq!(current_index(&lines, 99.0), Some(1));
    }

    #[test]
    fn cleans_streaming_titles() {
        assert_eq!(clean_title("Song Name (Official Video) [HD]"), "Song Name");
        let t = Track {
            title: "M83 - Midnight City (Official Video)".into(),
            artist: "M83VEVO".into(),
            duration: 243.0,
        };
        assert_eq!(guess_names(&t), ("Midnight City".into(), "M83".into()));
        let t2 = Track {
            title: "Nightcall".into(),
            artist: "Kavinsky - Topic".into(),
            duration: 258.0,
        };
        assert_eq!(guess_names(&t2), ("Nightcall".into(), "Kavinsky".into()));
    }
}
