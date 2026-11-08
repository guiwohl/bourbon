use std::path::Path;
use std::time::{Duration, SystemTime};

use bourbon_core::{Appearance, NoteStore, Settings, ThemeSource};
use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Hsla, TestAppContext, VisualTestContext, point, px,
};

use super::{click, isolated_locations, open_view, open_view_with};
use crate::locations::Locations;
use crate::theme::BOURBON_LIGHT_BACKGROUND;
use crate::workspace::Workspace;

const LIGHT_SCHEME: &str = "background = \"#fdf6e3\"\nforeground = \"#586e75\"\naccent = \"#268bd2\"\nselection = \"#93a1a1\"\ncolor1 = \"#dc322f\"\ncolor8 = \"#839496\"\n";
const DARK_SCHEME: &str = "background = \"#001c2b\"\nforeground = \"#6c7c81\"\naccent = \"#268bd2\"\nselection = \"#6c7c81\"\ncolor1 = \"#d8322f\"\ncolor8 = \"#5f6f74\"\n";

fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, keystrokes: &str) {
    cx.update_window(handle, |_, window, cx| window.render_frame(cx))
        .unwrap();
    VisualTestContext::from_window(handle, cx).simulate_keystrokes(keystrokes);
}

const TAMPERED: u32 = 0x00ff_0000;

fn tamper_with_the_background(cx: &mut TestAppContext) {
    cx.update(|cx| Theme::update(cx, |theme| theme.colors.background = color(TAMPERED)));
}

fn background(cx: &mut TestAppContext) -> Hsla {
    cx.read(|cx| cx.theme().background)
}

fn color(hex: u32) -> Hsla {
    Hsla::from(gpui_kit::rgb(hex))
}

fn saved(locations: &Locations) -> Settings {
    locations.settings.load()
}

fn is_open(cx: &mut TestAppContext, view: &Entity<Workspace>) -> bool {
    view.read_with(cx, |view, _| view.settings_open)
}

fn locations_with_scheme(scheme: &str) -> Locations {
    let locations = isolated_locations();
    std::fs::write(&locations.omarchy_colors, scheme).unwrap();
    locations
}

fn copy_of(locations: &Locations) -> Locations {
    Locations {
        settings: locations.settings.clone(),
        omarchy_colors: locations.omarchy_colors.clone(),
        database: locations.database.clone(),
        backup_token: locations.backup_token.clone(),
        backup_work_dir: locations.backup_work_dir.clone(),
        backup_runner: locations.backup_runner.clone(),
    }
}

fn set_modified(path: &Path, seconds: u64) {
    let file = std::fs::File::options().write(true).open(path).unwrap();
    file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
        .unwrap();
}

#[gpui_kit::test]
fn ctrl_comma_opens_the_settings_view(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, NoteStore::open_in_memory().unwrap());
    assert!(!is_open(cx, &view));

    press(cx, handle, "ctrl-,");

    assert!(is_open(cx, &view));
}

#[gpui_kit::test]
fn the_open_settings_pane_action_opens_the_settings_view(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, NoteStore::open_in_memory().unwrap());

    press(cx, handle, "ctrl-p");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.input("open settings", cx);
    })
    .unwrap();
    press(cx, handle, "enter");

    assert!(is_open(cx, &view));
}

#[gpui_kit::test]
fn the_back_button_closes_the_settings_view(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, NoteStore::open_in_memory().unwrap());
    press(cx, handle, "ctrl-,");

    click(cx, handle, "settings-close");

    assert!(!is_open(cx, &view));
}

#[gpui_kit::test]
fn the_app_starts_in_the_light_bourbon_theme_by_default(cx: &mut TestAppContext) {
    open_view(cx, NoteStore::open_in_memory().unwrap());

    assert_eq!(background(cx), color(BOURBON_LIGHT_BACKGROUND));
    assert!(!cx.read(|cx| cx.theme().is_dark()));
}

#[gpui_kit::test]
fn choosing_light_applies_the_light_palette_and_saves(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let (handle, _view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );
    press(cx, handle, "ctrl-,");

    click(cx, handle, "settings-appearance-light");

    assert!(!cx.read(|cx| cx.theme().is_dark()));
    assert_eq!(saved(&locations).appearance, Appearance::Light);
}

#[gpui_kit::test]
fn saved_settings_are_applied_at_startup(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let mut stored = Settings::default();
    stored.appearance = Appearance::Light;
    stored.sidebar_visible = false;
    stored.set_sidebar_width(300);
    locations.settings.save(&stored).unwrap();

    let (_, view) = open_view_with(cx, NoteStore::open_in_memory().unwrap(), locations);

    assert!(!cx.read(|cx| cx.theme().is_dark()));
    view.read_with(cx, |view, _| {
        assert!(!view.sidebar.is_visible);
        assert_eq!(view.settings.sidebar_width(), 300);
    });
}

#[gpui_kit::test]
fn following_omarchy_applies_its_colors_and_saves(cx: &mut TestAppContext) {
    let locations = locations_with_scheme(LIGHT_SCHEME);
    let (handle, _view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );

    press(cx, handle, "ctrl-,");
    click(cx, handle, "settings-theme-omarchy");

    assert_eq!(background(cx), color(0x00fd_f6e3));
    assert!(!cx.read(|cx| cx.theme().is_dark()));
    assert_eq!(saved(&locations).theme_source, ThemeSource::Omarchy);
}

#[gpui_kit::test]
fn the_appearance_choice_is_ignored_while_following_omarchy(cx: &mut TestAppContext) {
    let locations = locations_with_scheme(DARK_SCHEME);
    let (handle, view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );
    press(cx, handle, "ctrl-,");
    click(cx, handle, "settings-theme-omarchy");

    click(cx, handle, "settings-appearance-dark");

    assert!(cx.read(|cx| cx.theme().is_dark()));
    view.read_with(cx, |view, _| {
        assert_eq!(view.settings.appearance, Appearance::Light);
    });
}

#[gpui_kit::test]
fn an_unreadable_omarchy_file_falls_back_to_bourbon_without_an_error(cx: &mut TestAppContext) {
    let locations = locations_with_scheme("not a scheme");
    let (handle, view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );

    press(cx, handle, "ctrl-,");
    click(cx, handle, "settings-theme-omarchy");

    assert_eq!(background(cx), color(BOURBON_LIGHT_BACKGROUND));
    view.read_with(cx, |view, _| assert!(view.error_message.is_none()));
}

#[gpui_kit::test]
fn a_changed_omarchy_file_is_reapplied_within_two_seconds(cx: &mut TestAppContext) {
    let locations = locations_with_scheme(DARK_SCHEME);
    set_modified(&locations.omarchy_colors, 1_000);
    let mut stored = Settings::default();
    stored.theme_source = ThemeSource::Omarchy;
    locations.settings.save(&stored).unwrap();
    open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );
    assert_eq!(background(cx), color(0x0000_1c2b));

    std::fs::write(&locations.omarchy_colors, LIGHT_SCHEME).unwrap();
    set_modified(&locations.omarchy_colors, 2_000);
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();

    assert_eq!(background(cx), color(0x00fd_f6e3));
}

#[gpui_kit::test]
fn toggling_the_sidebar_is_saved(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let (handle, _view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );

    press(cx, handle, "ctrl-/");
    assert!(!saved(&locations).sidebar_visible);

    press(cx, handle, "ctrl-/");
    assert!(saved(&locations).sidebar_visible);
}

#[gpui_kit::test]
fn resizing_clamps_the_widths_and_saves_only_when_the_drag_ends(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let (_, view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );

    view.update(cx, |view, cx| {
        view.resize_sidebar(5., cx);
        view.resize_note_list(5000., cx);
    });
    view.read_with(cx, |view, _| {
        assert_eq!(view.settings.sidebar_width(), 170);
        assert_eq!(view.settings.note_list_width(), 500);
    });
    assert_eq!(saved(&locations).sidebar_width(), 200);

    view.update(cx, Workspace::finish_resizing);
    assert_eq!(saved(&locations).sidebar_width(), 170);
    assert_eq!(saved(&locations).note_list_width(), 500);
}

fn drag_splitter(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str, distance: f32) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let start = window.find(id.to_owned()).bounds().center();
        window.drag(start, point(start.x + px(distance), start.y), cx);
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

#[gpui_kit::test]
fn dragging_the_sidebar_splitter_resizes_the_sidebar_within_its_limits_and_saves_on_release(
    cx: &mut TestAppContext,
) {
    let locations = isolated_locations();
    let (handle, view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );

    drag_splitter(cx, handle, "sidebar-splitter", 60.);
    let widened = view.read_with(cx, |view, _| view.settings.sidebar_width());
    assert!((201..=260).contains(&widened));
    assert_eq!(saved(&locations).sidebar_width(), widened);

    drag_splitter(cx, handle, "sidebar-splitter", 1000.);
    view.read_with(cx, |view, _| assert_eq!(view.settings.sidebar_width(), 500));
    assert_eq!(saved(&locations).sidebar_width(), 500);

    drag_splitter(cx, handle, "sidebar-splitter", -1000.);
    view.read_with(cx, |view, _| assert_eq!(view.settings.sidebar_width(), 170));
    assert_eq!(saved(&locations).sidebar_width(), 170);
}

#[gpui_kit::test]
fn dragging_the_note_list_splitter_resizes_only_the_note_list(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let (handle, view) = open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );

    drag_splitter(cx, handle, "note-list-splitter", 40.);
    let widened = view.read_with(cx, |view, _| {
        assert_eq!(view.settings.sidebar_width(), 200);
        view.settings.note_list_width()
    });
    assert!((261..=300).contains(&widened));
    assert_eq!(saved(&locations).note_list_width(), widened);

    drag_splitter(cx, handle, "note-list-splitter", -1000.);
    view.read_with(cx, |view, _| {
        assert_eq!(view.settings.note_list_width(), 220);
    });
}

#[gpui_kit::test]
fn the_theme_is_not_repainted_while_the_omarchy_file_is_unchanged(cx: &mut TestAppContext) {
    let locations = locations_with_scheme(DARK_SCHEME);
    set_modified(&locations.omarchy_colors, 1_000);
    let mut stored = Settings::default();
    stored.theme_source = ThemeSource::Omarchy;
    locations.settings.save(&stored).unwrap();
    open_view_with(cx, NoteStore::open_in_memory().unwrap(), locations);
    tamper_with_the_background(cx);

    cx.executor().advance_clock(Duration::from_secs(4));
    cx.run_until_parked();

    assert_eq!(background(cx), color(TAMPERED));
}

#[gpui_kit::test]
fn a_changed_omarchy_file_is_ignored_while_bourbon_is_the_theme_source(cx: &mut TestAppContext) {
    let locations = locations_with_scheme(DARK_SCHEME);
    set_modified(&locations.omarchy_colors, 1_000);
    open_view_with(
        cx,
        NoteStore::open_in_memory().unwrap(),
        copy_of(&locations),
    );
    tamper_with_the_background(cx);

    std::fs::write(&locations.omarchy_colors, LIGHT_SCHEME).unwrap();
    set_modified(&locations.omarchy_colors, 2_000);
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();

    assert_eq!(background(cx), color(TAMPERED));
}

#[gpui_kit::test]
fn opening_a_note_leaves_the_settings_view(cx: &mut TestAppContext) {
    let (store, seeded) = super::seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-,");

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| view.select(&seeded.older, window, cx));
    })
    .unwrap();

    assert!(!is_open(cx, &view));
}

#[gpui_kit::test]
fn the_view_mode_is_saved_when_it_changes(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let (store, _) = super::seeded_store();
    let (handle, _view) = open_view_with(cx, store, copy_of(&locations));

    click(cx, handle, "view-mode-preview");

    assert_eq!(
        saved(&locations).editor_view_mode,
        bourbon_core::ViewMode::Preview
    );
}

#[gpui_kit::test]
fn the_saved_view_mode_is_restored_at_startup(cx: &mut TestAppContext) {
    let locations = isolated_locations();
    let mut stored = Settings::default();
    stored.editor_view_mode = bourbon_core::ViewMode::SideBySide;
    locations.settings.save(&stored).unwrap();

    let (_, view) = open_view_with(cx, NoteStore::open_in_memory().unwrap(), locations);

    assert_eq!(
        view.read_with(cx, |view, _| view.view_mode),
        bourbon_core::ViewMode::SideBySide
    );
}
