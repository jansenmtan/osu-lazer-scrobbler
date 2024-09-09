use std::{
    error::Error,
    time::{Duration, SystemTime},
};
use tokio;

const API_KEY: &str = "17f5b398f790dc6bd0d4f798c5a40012";
const API_SECRET: &str = "fe0fd6aac0fb21162b78d6298d4f1f43";
const USERNAME: &str = "TheWhishkey";
const PASSWORD: &str = "I am an OG. 2014";

pub(crate) struct Scrobbler {
    scrobbler: rustfm_scrobble::Scrobbler,
    now_playing_track: Option<rustfm_scrobble::Scrobble>,
}

fn time_elapsed_since_playing(scrobble: &rustfm_scrobble::Scrobble) -> Option<u64> {
    let start_time = scrobble
        .as_map()
        .get("timestamp")
        .map(|s| s.as_str().parse::<u64>().unwrap());
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let elapsed = start_time.map(|start_time| now - start_time);
    elapsed
}

impl Scrobbler {
    pub(crate) fn new() -> Result<Self, Box<dyn Error>> {
        // TODO: use env vars or something else rather than hardcoding credentials
        let mut scrobbler = rustfm_scrobble::Scrobbler::new(API_KEY, API_SECRET);

        scrobbler.authenticate_with_password(USERNAME, PASSWORD)?;

        Ok(Self {
            scrobbler,
            now_playing_track: None,
        })
    }

    pub(crate) fn now_playing(&mut self, artist: String, track: String, album: String) -> Result<(), ()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut _track = rustfm_scrobble::Scrobble::new(artist.as_str(), track.as_str(), album.as_str());
        let track = _track.with_timestamp(timestamp);
        self.now_playing_track = Some(track.clone());
        match self.scrobbler.now_playing(&track) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    // scrobbles a currently playing track
    pub(crate) async fn scrobble(&self) -> Result<(), ()> {
        let scrobble_condition = |elapsed: u64, threshold: u64| elapsed >= threshold;
        const THRESHOLD: u64 = 25; // 25 sec


        match &self.now_playing_track {
            None => Err(()), // TODO: more useful errors
            Some(track) => {
                let previous_now_playing_track = track.clone();
                let elapsed = time_elapsed_since_playing(track).unwrap();

                loop {
                    if self.now_playing_track.as_ref().map_or(false, |t| &previous_now_playing_track == t) && scrobble_condition(elapsed, THRESHOLD) {
                        return match self.scrobbler.scrobble(track) {
                            Ok(_) => {
                                println!("SCROBBLER: SCROBBLED TRACK");
                                Ok(())
                            },
                            Err(_) => Err(()), // TODO: more useful errors
                        };
                    } else {
                        println!("SCROBBLER: SLEEPING FOR {}s", THRESHOLD - elapsed);
                        tokio::time::sleep(Duration::new(THRESHOLD - elapsed, 500_000_000)).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_raw_scrobble() {
        let scr = Scrobbler::new().unwrap();

        let artist = "Down";
        let track = "Rihan Rider";
        let album = "";
        let track = rustfm_scrobble::Scrobble::new(artist, track, album);

        let result = scr.scrobbler.scrobble(&track);
        dbg!(&result);

        assert!(result.is_ok());
    }
}

