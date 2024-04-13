use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct LyricsLine {
    pub seconds: i64,
    pub lyrics: String
}