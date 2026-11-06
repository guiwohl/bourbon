use bourbon_core::NoteStore;
use gpui_kit::{App, AppContext as _, Entity, KeyBinding, TitlebarOptions, Window, WindowOptions};

use crate::sidebar::ToggleSidebar;
use crate::workspace::{NewNote, Workspace};

const APP_ID: &str = "bourbon";
const WINDOW_TITLE: &str = "Bourbon";

pub fn run(store: NoteStore) {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            bind_keys(cx);
            let opened = gpui_kit::open_window(window_options(), cx, |window, cx| {
                build_workspace(store, window, cx)
            });
            if let Err(error) = opened {
                eprintln!("bourbon: cannot open the window: {error}");
                cx.quit();
            }
        });
}

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("ctrl-n", NewNote, None)]);
    cx.bind_keys([KeyBinding::new("ctrl-/", ToggleSidebar, None)]);
}

pub(crate) fn build_workspace(
    store: NoteStore,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Workspace> {
    let view = cx.new(|cx| Workspace::new(store, window, cx));
    let closing_view = view.downgrade();
    window.on_window_should_close(cx, move |_, cx| {
        closing_view
            .update(cx, Workspace::flush_before_closing)
            .unwrap_or(true)
    });
    view
}

fn window_options() -> WindowOptions {
    WindowOptions {
        app_id: Some(APP_ID.to_owned()),
        titlebar: Some(TitlebarOptions {
            title: Some(WINDOW_TITLE.into()),
            ..TitlebarOptions::default()
        }),
        ..WindowOptions::default()
    }
}

#[cfg(test)]
mod tests {
    use super::window_options;

    #[test]
    fn window_identifies_itself_as_bourbon() {
        let options = window_options();
        assert_eq!(options.app_id.as_deref(), Some("bourbon"));
        let title = options.titlebar.and_then(|titlebar| titlebar.title);
        assert_eq!(title.as_deref(), Some("Bourbon"));
    }
}
