use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use bourbon_core::NoteStore;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Entity, KeyBinding, TitlebarOptions, Window,
    WindowOptions,
};

use crate::editor::actions::{
    CycleViewMode, ExitFocusMode, FindInNote, HistoryBack, HistoryForward, LinkPickCancel,
    LinkPickConfirm, LinkPickNext, LinkPickPrevious, OpenNotebookSelector, ReplaceInNote,
    SetStatusActive, SetStatusCompleted, SetStatusDropped, SetStatusNone, SetStatusOnHold,
    ToggleFocusMode, ToggleTaskAtCursor,
};
use crate::fonts;
use crate::locations::Locations;
use crate::note_list::ClearFilter;
use crate::sidebar::ToggleSidebar;
use crate::workspace::{NewNote, OpenActionSearch, OpenNoteSearch, OpenSettings, Workspace};

struct Identity {
    app_id: &'static str,
    title: &'static str,
}

const RELEASE: Identity = Identity {
    app_id: "bourbon",
    title: "Bourbon",
};

const DEVELOPMENT: Identity = Identity {
    app_id: "bourbon-dev",
    title: "Bourbon Dev",
};

const CURRENT: Identity = if cfg!(feature = "dev") {
    DEVELOPMENT
} else {
    RELEASE
};

const FOCUS_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn app_id() -> &'static str {
    CURRENT.app_id
}

pub(crate) fn run(store: NoteStore, locations: Locations, focus_requests: Receiver<()>) {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            fonts::register(cx);
            bind_keys(cx);
            let opened = gpui_kit::open_window(window_options(&CURRENT), cx, |window, cx| {
                build_workspace(store, locations, window, cx)
            });
            match opened {
                Ok((window, _)) => forward_focus_requests(window, focus_requests, cx),
                Err(error) => {
                    eprintln!("bourbon: cannot open the window: {error}");
                    cx.quit();
                }
            }
        });
}

fn forward_focus_requests(window: AnyWindowHandle, focus_requests: Receiver<()>, cx: &mut App) {
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(FOCUS_POLL_INTERVAL).await;
            match focus_requests.try_recv() {
                Ok(()) => {
                    let _ = window.update(cx, |_, window, _| window.activate_window());
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => break,
            }
        }
    })
    .detach();
}

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-n", NewNote, None),
        KeyBinding::new("ctrl-/", ToggleSidebar, None),
        KeyBinding::new("ctrl-,", OpenSettings, None),
        KeyBinding::new("ctrl-o", OpenNoteSearch, None),
        KeyBinding::new("ctrl-p", OpenActionSearch, None),
        KeyBinding::new("shift-alt-1", SetStatusNone, None),
        KeyBinding::new("shift-alt-2", SetStatusActive, None),
        KeyBinding::new("shift-alt-3", SetStatusOnHold, None),
        KeyBinding::new("shift-alt-4", SetStatusCompleted, None),
        KeyBinding::new("shift-alt-5", SetStatusDropped, None),
        KeyBinding::new("ctrl-alt-m", OpenNotebookSelector, None),
        KeyBinding::new("alt-left", HistoryBack, None),
        KeyBinding::new("alt-right", HistoryForward, None),
        KeyBinding::new("ctrl-e", CycleViewMode, None),
        KeyBinding::new("ctrl-e", CycleViewMode, Some("Input")),
        KeyBinding::new("ctrl-f", FindInNote, None),
        KeyBinding::new("ctrl-h", ReplaceInNote, None),
        KeyBinding::new("ctrl-shift-f", ToggleFocusMode, None),
        KeyBinding::new("ctrl-shift-f", ToggleFocusMode, Some("Input")),
        KeyBinding::new("escape", ExitFocusMode, None),
        KeyBinding::new(
            "ctrl-enter",
            ToggleTaskAtCursor,
            Some("Input && mode == editor"),
        ),
        KeyBinding::new("up", LinkPickPrevious, Some("LinkPicker > Input")),
        KeyBinding::new("down", LinkPickNext, Some("LinkPicker > Input")),
        KeyBinding::new("enter", LinkPickConfirm, Some("LinkPicker > Input")),
        KeyBinding::new("escape", LinkPickCancel, Some("LinkPicker > Input")),
        KeyBinding::new("escape", ClearFilter, Some("NoteFilter > Input")),
    ]);
}

pub(crate) fn build_workspace(
    store: NoteStore,
    locations: Locations,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Workspace> {
    let view = cx.new(|cx| Workspace::new(store, locations, window, cx));
    let closing_view = view.downgrade();
    window.on_window_should_close(cx, move |window, cx| {
        closing_view
            .update(cx, |view, cx| view.request_close(window, cx))
            .unwrap_or(true)
    });
    view
}

fn window_options(identity: &Identity) -> WindowOptions {
    WindowOptions {
        app_id: Some(identity.app_id.to_owned()),
        titlebar: Some(TitlebarOptions {
            title: Some(identity.title.into()),
            ..TitlebarOptions::default()
        }),
        ..WindowOptions::default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use gpui_kit::{AppContext as _, Empty, TestAppContext, WindowOptions};

    use super::{
        DEVELOPMENT, FOCUS_POLL_INTERVAL, RELEASE, app_id, forward_focus_requests, window_options,
    };

    #[test]
    fn app_id_is_one_of_the_known_identities() {
        assert!([RELEASE.app_id, DEVELOPMENT.app_id].contains(&app_id()));
    }

    #[gpui_kit::test]
    fn a_focus_request_activates_the_window(cx: &mut TestAppContext) {
        let (sender, receiver) = mpsc::channel();
        let target = cx.update(|cx| {
            let target = cx
                .open_window(WindowOptions::default(), |_, cx| cx.new(|_| Empty))
                .unwrap();
            forward_focus_requests(target.into(), receiver, cx);
            target
        });
        assert_eq!(cx.update(|cx| cx.active_window()), None);
        sender.send(()).unwrap();
        cx.executor().advance_clock(FOCUS_POLL_INTERVAL * 2);
        cx.run_until_parked();
        assert_eq!(cx.update(|cx| cx.active_window()), Some(target.into()));
    }

    #[test]
    fn release_window_identifies_itself_as_bourbon() {
        let options = window_options(&RELEASE);
        assert_eq!(options.app_id.as_deref(), Some("bourbon"));
        let title = options.titlebar.and_then(|titlebar| titlebar.title);
        assert_eq!(title.as_deref(), Some("Bourbon"));
    }

    #[test]
    fn development_window_never_passes_for_the_real_app() {
        let options = window_options(&DEVELOPMENT);
        assert_eq!(options.app_id.as_deref(), Some("bourbon-dev"));
        let title = options.titlebar.and_then(|titlebar| titlebar.title);
        assert_eq!(title.as_deref(), Some("Bourbon Dev"));
    }
}
