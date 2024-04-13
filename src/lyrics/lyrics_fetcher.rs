use crate::lyrics::lyrics_line::LyricsLine;

#[derive(Default)]
pub struct LyricsFetcher {}

impl LyricsFetcher {
    pub fn fetch_lyrics(&self, song_name: &str, artist_name: &str) -> Option<Vec<LyricsLine>> {
        let request = ureq::get("https://prv.textyl.co/api/lyrics?pleasedontusethisapi=inotherprojects")
            .query("name", song_name)
            .query("artist", artist_name);

        return match request.call() {
            Ok(response) => {
                match response.into_json::<Vec<LyricsLine>>() {
                    Ok(lyrics_lines) => Some(lyrics_lines),
                    Err(_) => None
                }
            },
            Err(_) => None
        }
    }
}