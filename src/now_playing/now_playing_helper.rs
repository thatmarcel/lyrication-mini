use std::mem::transmute;

use block2::RcBlock;
use core_foundation::bundle::CFBundle;
use core_foundation::date::CFAbsoluteTimeGetCurrent;
use core_foundation::string::CFString;
use core_foundation::url::CFURL;
use crossbeam::channel::Sender;
use dispatch2::ffi::dispatch_queue_global_t;
use icrate::Foundation::{NSDictionary, NSString};
use objc2::runtime::NSObject;
use crate::now_playing::now_playing_info::NowPlayingInfo;

use crate::now_playing::ns_dictionary_extensions::NSDictionaryExtensions;

#[allow(improper_ctypes_definitions)]
type MRMediaRemoteGetNowPlayingInfoFunction = extern "C" fn(dispatch_queue_global_t, RcBlock<(*const NSDictionary<NSString, NSObject>,), ()>);

pub struct NowPlayingHelper {
    get_now_playing_info_function: MRMediaRemoteGetNowPlayingInfoFunction
}

impl Default for NowPlayingHelper {
    fn default() -> Self {
        unsafe {
            let bundle_url = CFURL::from_path(
                "/System/Library/PrivateFrameworks/MediaRemote.framework",
                true
            ).unwrap();
            let bundle = CFBundle::new(bundle_url).unwrap();

            NowPlayingHelper {
                get_now_playing_info_function: transmute(bundle.function_pointer_for_name(
                    CFString::from_static_string("MRMediaRemoteGetNowPlayingInfo")
                ))
            }
        }
    }
}

impl NowPlayingHelper {
    pub fn get_now_playing_song_info(&self, tx: Sender<NowPlayingInfo>) {
        unsafe {
            self.unsafe_get_now_playing_song_info(tx);
        }
    }

    unsafe fn unsafe_get_now_playing_song_info(&self, tx: Sender<NowPlayingInfo>) {
        let callback_block = block2::ConcreteBlock::new(move |raw_info_dictionary: *const NSDictionary<NSString, NSObject>| {
            let info_dictionary = match raw_info_dictionary.as_ref() {
                Some(x) => x,
                None => return
            };

            let song_name: Option<String> = info_dictionary.get_string_for_key("kMRMediaRemoteNowPlayingInfoTitle");
            let artist_name: Option<String> = info_dictionary.get_string_for_key("kMRMediaRemoteNowPlayingInfoArtist");

            let playback_rate = info_dictionary.get_f64_for_key("kMRMediaRemoteNowPlayingInfoPlaybackRate").unwrap_or_default();
            let is_playing = playback_rate > 0f64;

            let song_progress_until_previous_pause: f64 = info_dictionary.get_f64_for_key("kMRMediaRemoteNowPlayingInfoElapsedTime").unwrap_or_default();
            let song_progress = match is_playing {
                true => {
                    let last_play_event_timestamp: f64 = info_dictionary.get_absolute_date_time_for_key("kMRMediaRemoteNowPlayingInfoTimestamp").unwrap_or_default();
                    let time_interval_since_last_play_event: f64 = CFAbsoluteTimeGetCurrent() - last_play_event_timestamp;

                    time_interval_since_last_play_event + song_progress_until_previous_pause
                },
                false => song_progress_until_previous_pause
            };

            tx.send(NowPlayingInfo {
                song_name,
                artist_name,
                is_playing,
                song_progress_in_seconds: song_progress
            }).unwrap_or_default();
        });

        let dispatch_queue = dispatch2::ffi::dispatch_get_global_queue(0, 0);

        let _ = &(self.get_now_playing_info_function)(
            dispatch_queue,
            callback_block.copy()
        );
    }
}