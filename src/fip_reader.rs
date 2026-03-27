use crate::track::Track;
use crate::TwifipError;
use log::{error, info};
use serde::Deserialize;

pub type Result<T> = std::result::Result<T, TwifipError>;

#[derive(Deserialize, Debug)]
struct Release {
    title: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Song {
    release: Option<Release>,
}

#[derive(Deserialize, Debug)]
struct Now {
    #[serde(rename = "firstLine")]
    first_line: String,
    #[serde(rename = "secondLine")]
    second_line: String,
    song: Option<Song>,
    #[serde(rename = "startTime")]
    start_time: i64,
}

#[derive(Deserialize, Debug)]
struct Fip {
    now: Now,
}

/// Resolve a devalue-encoded array into a plain serde_json::Value.
///
/// Devalue (used by SvelteKit) encodes an object graph as a flat array where
/// integer values inside objects and arrays are references (indices) into that
/// same array. Primitives (strings, numbers, booleans, null) stored in the
/// array are used as-is.
fn devalue_resolve(arr: &[serde_json::Value], idx: usize) -> serde_json::Value {
    match &arr[idx] {
        serde_json::Value::Object(map) => {
            let mut resolved = serde_json::Map::new();
            for (k, v) in map {
                let v_resolved = if let Some(i) = v.as_u64() {
                    devalue_resolve(arr, i as usize)
                } else {
                    v.clone()
                };
                resolved.insert(k.clone(), v_resolved);
            }
            serde_json::Value::Object(resolved)
        }
        serde_json::Value::Array(inner) => {
            let resolved: Vec<serde_json::Value> = inner
                .iter()
                .map(|v| {
                    if let Some(i) = v.as_u64() {
                        devalue_resolve(arr, i as usize)
                    } else {
                        v.clone()
                    }
                })
                .collect();
            serde_json::Value::Array(resolved)
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fip_json_deserialization() {
        // Devalue-encoded response: integers in objects/arrays are array indices.
        // Index 0: root {now->1, next->6, delayToRefresh->7}
        // Index 1: now {startTime->2, firstLine->3, secondLine->4, song->5}
        // Index 2: 1740168451 (startTime literal)
        // Index 3: "Some Title"
        // Index 4: "Some Artist"
        // Index 5: null (no song)
        // Index 6: [] (next)
        // Index 7: 20000 (delayToRefresh literal)
        let result_array = r#"[
            {"now": 1, "next": 6, "delayToRefresh": 7},
            {"startTime": 2, "firstLine": 3, "secondLine": 4, "song": 5},
            1740168451,
            "Some Title",
            "Some Artist",
            null,
            [],
            20000
        ]"#;

        let arr: Vec<serde_json::Value> = serde_json::from_str(result_array).unwrap();
        let resolved = devalue_resolve(&arr, 0);
        let fip: Fip = serde_json::from_value(resolved).unwrap();

        assert_eq!(fip.now.first_line, "Some Title");
        assert_eq!(fip.now.second_line, "Some Artist");
        assert!(fip.now.song.is_none());
        assert_eq!(fip.now.start_time, 1740168451);
    }

    #[test]
    fn test_fip_json_deserialization_with_song() {
        // Index 0: root
        // Index 1: now
        // Index 2: startTime
        // Index 3: "Some Title"
        // Index 4: "Some Artist"
        // Index 5: song {release->6}
        // Index 6: release {title->7}
        // Index 7: "Some Album"
        // Index 8: [] (next)
        // Index 9: 20000
        let result_array = r#"[
            {"now": 1, "next": 8, "delayToRefresh": 9},
            {"startTime": 2, "firstLine": 3, "secondLine": 4, "song": 5},
            1740168451,
            "Some Title",
            "Some Artist",
            {"release": 6},
            {"title": 7},
            "Some Album",
            [],
            20000
        ]"#;

        let arr: Vec<serde_json::Value> = serde_json::from_str(result_array).unwrap();
        let resolved = devalue_resolve(&arr, 0);
        let fip: Fip = serde_json::from_value(resolved).unwrap();

        assert_eq!(fip.now.first_line, "Some Title");
        assert_eq!(fip.now.second_line, "Some Artist");
        assert_eq!(
            fip.now.song.unwrap().release.unwrap().title.unwrap(),
            "Some Album"
        );
        assert_eq!(fip.now.start_time, 1740168451);
    }
}

pub(crate) fn read_fip() -> Result<Track> {
    let url = "https://www.radiofrance.fr/_app/remote/di23tz/getLive?payload=W3siYnJhbmROYW1lIjoxfSwiZmlwIl0";
    let response = attohttpc::get(url).send()?;
    if response.is_success() {
        let text = response.text_utf8()?;
        let outer: serde_json::Value = serde_json::from_str(&text)?;
        let result_str = outer["result"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'result' field in FIP response"))?;
        let arr: Vec<serde_json::Value> = serde_json::from_str(result_str)?;
        let resolved = devalue_resolve(&arr, 0);
        let fip: Fip = serde_json::from_value(resolved)?;
        info!("fip: {:?}", fip);
        let track: Option<Track> = Track::create(
            fip.now.second_line,
            fip.now.first_line,
            fip.now.song.and_then(|s| s.release).and_then(|r| r.title),
            fip.now.start_time,
        );
        track.ok_or_else(|| TwifipError::TrackFilterError("FIP response was not a music track".to_string()))
    } else {
        error!("Invalid response from get {} : {}", url, response.status());
        return Err(TwifipError::FipApiError(format!("Invalid response from FIP API: {}", response.status())));
    }
}
