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
    for sep in [" - ", " – ", " — ", " ‒ ", " | "] {
        if let Some((artist, song)) = title.split_once(sep) {
            return (song.trim().to_string(), artist.trim().to_string());
        }
    }
    let artist = track
        .artist
        .trim_end_matches(" - Topic")
        .trim_end_matches("VEVO")
        .trim()
        .to_string();
    (title, artist)
}

/// Fallback source (NetEase) for when LRCLIB is down or lacks the song.
fn fetch_netease(track: &Track) -> Option<Vec<Line>> {
    let (title, artist) = guess_names(track);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent("Mozilla/5.0")
        .build();
    let found: serde_json::Value = agent
        .get("https://music.163.com/api/search/get")
        .query("s", &format!("{title} {artist}"))
        .query("type", "1")
        .query("limit", "6")
        .call()
        .ok()?
        .into_json()
        .ok()?;
    let songs = found.get("result")?.get("songs")?.as_array()?;
    let artist_lc = artist.to_lowercase();
    // Prefer songs by the right artist and of about the right length.
    let mut ranked: Vec<(&serde_json::Value, bool, f64)> = songs
        .iter()
        .map(|s| {
            let by = s
                .get("artists")
                .and_then(|a| a.as_array())
                .is_some_and(|a| {
                    a.iter().any(|x| {
                        x.get("name").and_then(|n| n.as_str()).is_some_and(|n| {
                            let n = n.to_lowercase();
                            !artist_lc.is_empty()
                                && (artist_lc.contains(&n) || n.contains(&artist_lc))
                        })
                    })
                });
            let d = s.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0) / 1000.0;
            let diff = if track.duration > 0.0 && d > 0.0 {
                (d - track.duration).abs()
            } else {
                0.0
            };
            (s, by, diff)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.total_cmp(&b.2)));
    let norm = |s: &str| {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
    };
    let want = norm(&title);
    for (song, by_artist, diff) in ranked.into_iter().take(6) {
        // Wrong lyrics are worse than none: demand a title match, a matching artist
        // when one is known, and a similar length.
        let name = norm(song.get("name").and_then(|n| n.as_str()).unwrap_or(""));
        let title_ok =
            !want.is_empty() && !name.is_empty() && (name.contains(&want) || want.contains(&name));
        if !title_ok || (!artist_lc.is_empty() && !by_artist) || diff > 8.0 {
            continue;
        }
        let id = song.get("id")?.as_i64()?.to_string();
        let lyric: Option<serde_json::Value> = agent
            .get("https://music.163.com/api/song/lyric")
            .query("id", &id)
            .query("lv", "1")
            .call()
            .ok()
            .and_then(|r| r.into_json().ok());
        let lines = lyric
            .as_ref()
            .and_then(|v| v.get("lrc")?.get("lyric")?.as_str().map(parse_lrc))
            .filter(|l| !l.is_empty());
        if lines.is_some() {
            return lines;
        }
    }
    None
}

fn fetch(track: &Track) -> Option<Vec<Line>> {
    fetch_lrclib(track).or_else(|| fetch_netease(track))
}

fn fetch_lrclib(track: &Track) -> Option<Vec<Line>> {
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

    // Looser queries and duration tolerances; first hit with synced lyrics wins.
    let queries = [
        format!("{title} {artist}"),
        title.clone(),
        clean_title(&track.title),
    ];
    for q in queries.iter().filter(|q| !q.trim().is_empty()) {
        let Some(results) = agent
            .get("https://lrclib.net/api/search")
            .query("q", q)
            .call()
            .ok()
            .and_then(|r| r.into_json::<Vec<serde_json::Value>>().ok())
        else {
            continue;
        };
        let want = title.to_lowercase();
        for tolerance in [4.0, 10.0] {
            let hit = results.iter().find_map(|r| {
                let d = r.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
                if track.duration > 0.0 && d > 0.0 && (d - track.duration).abs() > tolerance {
                    return None;
                }
                let name = r
                    .get("trackName")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                if name.is_empty() || !(name.contains(&want) || want.contains(&name)) {
                    return None;
                }
                synced(r)
            });
            if hit.is_some() {
                return hit;
            }
        }
    }
    None
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
    busy: bool,
    failed_at: Option<std::time::Instant>,
}

/// Lyrics for the current track, fetched in the background.
pub struct Lyrics {
    slot: Arc<Mutex<Slot>>,
    ctx: egui::Context,
    enabled: bool,
}

pub struct LyricView {
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
                // Retry a failed lookup (network blip) after a pause.
                let retry = slot.lines.is_none()
                    && !slot.busy
                    && slot
                        .failed_at
                        .is_some_and(|t| t.elapsed() > Duration::from_secs(20));
                if !retry {
                    return;
                }
            }
            slot.key = key.clone();
            slot.lines = None;
            slot.busy = true;
            slot.failed_at = None;
        }
        let slot = Arc::clone(&self.slot);
        let ctx = self.ctx.clone();
        let track = track.clone();
        let spawned = std::thread::Builder::new()
            .name("umbra-lyrics".into())
            .spawn(move || {
                let lines = fetch(&track);
                if std::env::var_os("UMBRA_DEBUG_LYRICS").is_some() {
                    eprintln!(
                        "umbra: lyrics for {:?} / {:?} ({}s): {}",
                        track.title,
                        track.artist,
                        track.duration,
                        lines.as_ref().map_or(0, Vec::len)
                    );
                }
                let mut slot = slot.lock().unwrap();
                // Ignore the answer if the song changed while we were waiting.
                if slot.key == key {
                    slot.busy = false;
                    if lines.is_none() {
                        slot.failed_at = Some(std::time::Instant::now());
                    }
                    slot.lines = lines;
                    drop(slot);
                    ctx.request_repaint();
                }
            });
        if let Err(err) = spawned {
            eprintln!("umbra: lyrics thread failed: {err}");
        }
    }

    /// True once a lookup has finished without finding lyrics (a retry may follow).
    pub fn not_found(&self) -> bool {
        let slot = self.slot.lock().unwrap();
        !slot.busy && slot.lines.is_none() && slot.failed_at.is_some()
    }

    /// `before` lines, the current line, then `after` lines around `position`. Slots
    /// outside the song are `None`. Index `before` is the current line.
    pub fn window(
        &self,
        position: f64,
        before: usize,
        after: usize,
    ) -> Option<Vec<Option<String>>> {
        let slot = self.slot.lock().unwrap();
        let lines = slot.lines.as_ref()?;
        // Before the first line, treat the cursor as sitting just ahead of line 0.
        let cur = current_index(lines, position).map_or(-1, |i| i as isize);
        Some(
            (0..before + 1 + after)
                .map(|k| {
                    let i = cur + k as isize - before as isize;
                    usize::try_from(i)
                        .ok()
                        .and_then(|i| lines.get(i))
                        .map(|l| l.text.clone())
                })
                .collect(),
        )
    }

    /// Current and next lyric line at `position`, plus the time the next line starts.
    pub fn at(&self, position: f64) -> Option<LyricView> {
        let slot = self.slot.lock().unwrap();
        let lines = slot.lines.as_ref()?;
        let idx = current_index(lines, position);
        let next = idx.map_or(0, |i| i + 1);
        Some(LyricView {
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
        let t3 = Track {
            title: "Maroon 5, Wiz Khalifa – Payphone (Lyrics)".into(),
            artist: "Latin City".into(),
            duration: 231.0,
        };
        assert_eq!(
            guess_names(&t3),
            ("Payphone".into(), "Maroon 5, Wiz Khalifa".into())
        );
    }
}
