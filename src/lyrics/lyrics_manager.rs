use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crossbeam::channel::{Sender, unbounded};

use crate::lyrics::lyrics_fetcher::LyricsFetcher;
use crate::lyrics::lyrics_line::LyricsLine;
use crate::now_playing::now_playing_helper::NowPlayingHelper;
use crate::now_playing::now_playing_info::NowPlayingInfo;

pub struct LyricsManager {
    now_playing_helper: NowPlayingHelper,
    lyrics_fetcher: LyricsFetcher,
    lyrics_line_text_updated_tx: Sender<Option<String>>,
    previous_song_name: Option<String>,
    previous_artist_name: Option<String>,
    previous_lyrics_line: Option<LyricsLine>,
    current_lyrics_lines: Option<Vec<LyricsLine>>,
    is_fetching_lyrics: bool
}

static mut LYRICS_MANAGER: Option<LyricsManager> = None;

impl LyricsManager {
    pub fn new(lyrics_line_text_updated_tx: Sender<Option<String>>) {
        unsafe {
            thread::spawn(move || {
                let mut lyrics_manager = LyricsManager {
                    now_playing_helper: NowPlayingHelper::default(),
                    lyrics_fetcher: LyricsFetcher::default(),
                    lyrics_line_text_updated_tx,
                    previous_song_name: None,
                    previous_artist_name: None,
                    previous_lyrics_line: None,
                    current_lyrics_lines: None,
                    is_fetching_lyrics: false
                };

                lyrics_manager.start();

                LYRICS_MANAGER = Some(lyrics_manager);
            });
        }
    }

    pub fn start(&mut self) {
        loop {
            self.update();
            sleep(Duration::from_millis(200));
        }
    }

    fn update(&mut self) {
        if self.is_fetching_lyrics {
            return;
        }

        let (tx,rx) = unbounded();
        self.now_playing_helper.get_now_playing_song_info(tx);
        let now_playing_info = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(npi) => npi,
            Err(_) => return
        };

        if !now_playing_info.is_playing {
            self.lyrics_line_text_updated_tx.send(None).unwrap_or_default();
            return;
        }

        if now_playing_info.song_name != self.previous_song_name || now_playing_info.artist_name != self.previous_artist_name {
            self.previous_song_name = now_playing_info.song_name.clone();
            self.previous_artist_name = now_playing_info.artist_name.clone();

            self.handle_new_song(&now_playing_info);
        } else {
            self.handle_same_song_update(&now_playing_info);
        }
    }

    fn handle_new_song(&mut self, now_playing_info: &NowPlayingInfo) {
        let song_name = match &now_playing_info.song_name {
            Some(sn) => sn,
            None => {
                self.lyrics_line_text_updated_tx.send(None).unwrap_or_default();
                return;
            }
        };
        let artist_name = match &now_playing_info.artist_name {
            Some(an) => an,
            None => {
                self.lyrics_line_text_updated_tx.send(None).unwrap_or_default();
                return;
            }
        };

        self.is_fetching_lyrics = true;

        let lyrics_lines = self.lyrics_fetcher.fetch_lyrics(
            &song_name,
            &artist_name
        );

        self.current_lyrics_lines = lyrics_lines;

        self.is_fetching_lyrics = false;

        self.handle_same_song_update(now_playing_info);
    }

    fn handle_same_song_update(&mut self, now_playing_info: &NowPlayingInfo) {
        let mut current_lyrics_lines = match &self.current_lyrics_lines {
            Some(cll) => cll.clone(),
            None => {
                self.lyrics_line_text_updated_tx.send(None).unwrap_or_default();
                return;
            }
        };

        current_lyrics_lines.sort_by(|a, b| {
            (now_playing_info.song_progress_in_seconds - (b.seconds as f64)).abs()
                .partial_cmp(&(now_playing_info.song_progress_in_seconds - (a.seconds as f64)).abs())
                .unwrap()
        });

        match current_lyrics_lines
            .iter()
            // .filter(|line| { line.seconds < (now_playing_info.song_progress_in_seconds + 1.0) as i64 })
            .last() {
            Some(current_lyrics_line) => {
                if self.previous_lyrics_line.is_some() && *current_lyrics_line == *self.previous_lyrics_line.as_ref().unwrap() {
                    return;
                }

                self.previous_lyrics_line = Some(current_lyrics_line.clone());
                self.lyrics_line_text_updated_tx.send(Some(current_lyrics_line.lyrics.clone())).unwrap_or_default();
            },
            None => return
        };
    }
}