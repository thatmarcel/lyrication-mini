use cacao::appkit::{App};
use crate::app::LyricationMiniApp;

mod now_playing;
mod app;
mod lyrics;

fn main() {
    App::new("com.thatmarcel.apps.lyricationmini", LyricationMiniApp::default()).run();
}
