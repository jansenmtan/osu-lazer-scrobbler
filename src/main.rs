use regex::{Captures, Regex};
use rustfm_scrobble::Scrobble;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::sleep;

mod lookout;
mod scrobbler;

fn get_lazer_log_path() -> PathBuf {
    let mut pb = PathBuf::new();
    // TODO: detect path rather than use hardcoded path
    pb.push("D:\\osu-lazer\\logs");
    pb
}

fn get_runtime_path() -> PathBuf {
    let pattern = Regex::new(r"\d+\.runtime\.log").unwrap();
    let lazer_log_dir = fs::read_dir(Path::new(&get_lazer_log_path())).unwrap();
    let runtime_path = lazer_log_dir
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map_or(false, |name| pattern.is_match(name))
        })
        .max_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok())
        .map(|entry| entry.path())
        .unwrap();

    runtime_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_runtime_path() {
        let result = get_runtime_path();
        let pattern = Regex::new(r"\d+\.runtime\.log").unwrap();
        dbg!(result.clone());
        assert!(pattern.is_match(result.to_str().unwrap()))
    }
}

#[derive(PartialEq, Eq, Debug)]
struct BeatmapSet {
    artist: String,
    title: String,
    creator: String,
}

async fn watch_and_send_beatmapsets(tx: Sender<BeatmapSet>) {
    let mut lines = linemux::MuxedLines::new().unwrap();
    dbg!(&get_runtime_path());
    lines
        .add_file(Path::new(&get_runtime_path()))
        .await
        .unwrap();

    while let Ok(Some(line)) = lines.next_line().await {
        let pattern = Regex::new(r"^(?<date>\d{4}-\d{2}-\d{2}) (?<time_utc>\d{2}:\d{2}:\d{2}) \[.+\]: Song select working beatmap updated to (?<beatmap>(?<artist>.+) - (?<title>.+) \((?<creator>.+)\) \[(?<difficulty>.+)\])$").unwrap();
        let _captures = pattern.captures(line.line());

        if _captures.is_none() {
            continue; // yes, i am avoiding 1 (one) line indent
        }

        let captures = _captures.unwrap();
        let _get =
            |captures: &Captures, name: &str| captures.name(name).unwrap().as_str().to_string();
        let beatmapset = BeatmapSet {
            artist: _get(&captures, "artist"),
            title: _get(&captures, "title"),
            creator: _get(&captures, "creator"),
        };

        tx.send(beatmapset).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    // TODO: process monitoring
    //println!("Monitoring for osu!lazer...");

    // TODO: actually determine if osu!lazer is running
    println!("osu!lazer is running. Scrobbling...");

    let (tx, mut rx) = mpsc::channel(4);

    tokio::spawn(async move {
        watch_and_send_beatmapsets(tx).await;
    });

    use std::sync::{Arc, Mutex};
    let current_task: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    let scr_mgr = Arc::new(Mutex::new(scrobbler::Manager::new().unwrap()));

    while let Some(beatmapset) = rx.recv().await {
        println!("Received {:?}", beatmapset);

        let current_task = current_task.clone();
        let scr_mgr = scr_mgr.clone();

        // Spawn a new task for scrobbling
        let new_task = tokio::spawn(async move {
            // Wait for the player to actually listen to the track
            sleep(Duration::from_secs(25)).await;

            // Then scrobble
            let scrobble = Scrobble::new(beatmapset.artist.as_str(), beatmapset.title.as_str(), "");
            let scr_mgr = scr_mgr.lock().unwrap();
            match scr_mgr.scrobbler.scrobble(&scrobble) {
                Ok(_) => {
                    println!("Scrobbled {:?}", scrobble);
                }
                Err(e) => {
                    println!("Failed to scrobble: {:?}", e);
                }
            }
        });

        // Lock the current task and cancel the previous one if it exists
        let mut current = current_task.lock().unwrap();
        if let Some(task) = current.take() {
            task.abort(); // Cancel the previous task
        }
        *current = Some(new_task); // Replace with the new task
    }
}
