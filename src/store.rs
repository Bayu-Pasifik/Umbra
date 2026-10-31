//! Local persistence: one JSON file in the OS config directory. No network, ever.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// `<config dir>/umbra/data.json`
pub fn default_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("umbra").join("data.json"))
}

/// Load saved state. Missing or corrupt files yield an empty map, never an error:
/// a bad file must not stop the island from starting.
pub fn load_from(path: &Path) -> Map<String, Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| match v {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default()
}

/// Write state atomically (temp file, then rename) so a crash cannot leave half a file.
pub fn save_to(path: &Path, state: &Map<String, Value>) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_file(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("umbra-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn round_trips_state() {
        let dir = temp_file("rt");
        let path = dir.join("data.json");
        let mut state = Map::new();
        state.insert("water".into(), json!({"ml": 750}));
        save_to(&path, &state).unwrap();
        assert_eq!(load_from(&path), state);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_and_corrupt_files_load_empty() {
        let dir = temp_file("bad");
        let path = dir.join("data.json");
        assert!(load_from(&path).is_empty());
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "{ not json").unwrap();
        assert!(load_from(&path).is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}
