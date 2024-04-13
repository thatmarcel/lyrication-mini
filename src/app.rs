use std::ffi::c_void;
use std::intrinsics::transmute;
use std::thread;

use cacao::appkit::AppDelegate;
use cacao::appkit::window::{Window, WindowConfig, WindowStyle};
use cacao::color::Color;
use cacao::layout::Layout;
use cacao::objc::{msg_send, sel, sel_impl};
use cacao::text::{Font, Label};
use cacao::utils::async_main_thread;
use cacao::view::View;
use crossbeam::channel::unbounded;
use icrate::AppKit::{NSApplication, NSTextField};
use icrate::Foundation::{MainThreadMarker, NSString};

use crate::lyrics::lyrics_manager::LyricsManager;

pub struct LyricationMiniApp {
    window: Window,
    content_view: View,
    lyrics_line_label: Label
}

impl Default for LyricationMiniApp {
    fn default() -> Self {
        let mut window_config = WindowConfig::default();
        window_config.set_styles(&[
            WindowStyle::Borderless,
            WindowStyle::Resizable
        ]);

        let app = LyricationMiniApp {
            window: Window::new(window_config),
            content_view: View::new(),
            lyrics_line_label: Label::new()
        };

        app
    }
}

impl AppDelegate for LyricationMiniApp {
    fn did_finish_launching(&self) {
        self.window.set_content_size(400, 100);
        self.window.set_title("Lyrication Mini");
        self.window.set_movable_by_background(true);
        self.window.set_background_color(Color::Clear);

        self.content_view.set_background_color(Color::MacOSWindowBackgroundColor);
        self.content_view.layer.set_corner_radius(8f64);
        self.window.set_content_view(&self.content_view);

        self.window.show();

        let ns_window = self.window.objc.to_owned();

        unsafe {
            let _: c_void = msg_send![ns_window, setLevel: 1000i64];
        }

        self.window.make_key_and_order_front();

        self.setup_views();
    }
}

impl LyricationMiniApp {
    fn setup_views(&self) {
        self.lyrics_line_label.set_translates_autoresizing_mask_into_constraints(false);
        self.content_view.add_subview(&self.lyrics_line_label);
        self.lyrics_line_label.top.constraint_equal_to(&self.content_view.top).offset(16f64).set_active(true);
        self.lyrics_line_label.bottom.constraint_equal_to(&self.content_view.bottom).offset(-16f64).set_active(true);
        self.lyrics_line_label.left.constraint_equal_to(&self.content_view.left).offset(16f64).set_active(true);
        self.lyrics_line_label.right.constraint_equal_to(&self.content_view.right).offset(-16f64).set_active(true);
        self.lyrics_line_label.set_text_color(Color::Label);
        self.lyrics_line_label.set_font(Font::bold_system(16f64));
        self.lyrics_line_label.set_text("Loading...");

        let (tx, rx) = unbounded();

        LyricsManager::new(tx);

        thread::spawn(move || {
            loop {
                let line_text = rx.recv().unwrap_or_default();

                async_main_thread(move || unsafe {
                    let app = NSApplication::sharedApplication(MainThreadMarker::new_unchecked());
                    let windows = app.windows();
                    let window = match windows.first() {
                        Some(w) => w,
                        None => return
                    };
                    let content_view = match window.contentView() {
                        Some(cv) => cv,
                        None => return
                    };

                    match &line_text {
                        Some(lt) => {
                            content_view.setHidden(false);
                            let label: &NSTextField = transmute(match content_view.subviews().first() {
                                Some(l) => l,
                                None => return
                            });
                            label.setStringValue(&*NSString::from_str(lt.as_str()));
                        }
                        None => content_view.setHidden(true)
                    }
                });
            }
        });
    }
}