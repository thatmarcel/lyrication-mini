#[derive(Debug)]
pub struct NowPlayingInfo {
    pub song_name: Option<String>,
    pub artist_name: Option<String>,
    pub is_playing: bool,
    pub song_progress_in_seconds: f64
}