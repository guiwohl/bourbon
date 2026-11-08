use std::cell::RefCell;
use std::rc::Rc;

use bourbon_editor::{Editor, EditorEvent, EditorTheme, bind_editor_keys};
use gpui::{
    Entity, EntityInputHandler, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, ScrollDelta,
    ScrollEvent, ScrollSource, TestAppContext, VisualTestContext, point, px,
};

const CHAR_WIDTH: f32 = 7.8;
const LINE_HEIGHT: f32 = 19.5;
const TEXT_LEFT: f32 = 16.0;

fn open<'a>(cx: &'a mut TestAppContext, text: &str) -> (Entity<Editor>, &'a mut VisualTestContext) {
    cx.update(bind_editor_keys);
    let text = text.to_owned();
    let (editor, cx) =
        cx.add_window_view(move |_, cx| Editor::new(&text, EditorTheme::light(), cx));
    cx.update(|window, cx| {
        let handle = editor.read(cx).focus_handle().clone();
        window.focus(&handle, cx);
    });
    cx.run_until_parked();
    (editor, cx)
}

fn text(editor: &Entity<Editor>, cx: &mut VisualTestContext) -> String {
    editor.read_with(cx, |editor, _| editor.text())
}

fn selection(editor: &Entity<Editor>, cx: &mut VisualTestContext) -> std::ops::Range<usize> {
    editor.read_with(cx, |editor, _| editor.selected_range())
}

fn set_selection(editor: &Entity<Editor>, cx: &mut VisualTestContext, anchor: usize, head: usize) {
    editor.update(cx, |editor, cx| editor.set_selection(anchor, head, cx));
    cx.run_until_parked();
}

fn column_x(column: f32) -> f32 {
    TEXT_LEFT + column * CHAR_WIDTH + 1.0
}

fn row_y(row: f32) -> f32 {
    row * LINE_HEIGHT + 5.0
}

#[gpui::test]
fn typing_inserts_text(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "");
    cx.simulate_input("hi");
    assert_eq!(text(&editor, cx), "hi");
    assert_eq!(selection(&editor, cx), 2..2);
}

#[gpui::test]
fn typing_an_opening_bracket_inserts_the_pair(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "");
    cx.simulate_input("(");
    assert_eq!(text(&editor, cx), "()");
    assert_eq!(selection(&editor, cx), 1..1);
}

#[gpui::test]
fn composing_text_marks_it_until_committed(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "ab");
    set_selection(&editor, cx, 1, 1);
    editor.update_in(cx, |editor, window, cx| {
        editor.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
    });
    assert_eq!(text(&editor, cx), "anib");
    let marked = editor.update_in(cx, |editor, window, cx| {
        editor.marked_text_range(window, cx)
    });
    assert_eq!(marked, Some(1..3));
    editor.update_in(cx, |editor, window, cx| {
        editor.replace_text_in_range(None, "你", window, cx);
    });
    assert_eq!(text(&editor, cx), "a你b");
    let marked = editor.update_in(cx, |editor, window, cx| {
        editor.marked_text_range(window, cx)
    });
    assert_eq!(marked, None);
    assert_eq!(selection(&editor, cx), 4..4);
}

#[gpui::test]
fn input_handler_reports_utf16_offsets(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "a🙂b");
    set_selection(&editor, cx, 5, 5);
    let reported = editor.update_in(cx, |editor, window, cx| {
        editor
            .selected_text_range(false, window, cx)
            .map(|selection| selection.range)
    });
    assert_eq!(reported, Some(3..3));
    let slice = editor.update_in(cx, |editor, window, cx| {
        editor.text_for_range(1..3, &mut None, window, cx)
    });
    assert_eq!(slice.as_deref(), Some("🙂"));
}

#[gpui::test]
fn replacing_an_explicit_range_edits_there(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "hello");
    editor.update_in(cx, |editor, window, cx| {
        editor.replace_text_in_range(Some(1..3), "EY", window, cx);
    });
    assert_eq!(text(&editor, cx), "hEYlo");
}

#[gpui::test]
fn arrows_move_and_shift_extends(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "abc def");
    set_selection(&editor, cx, 0, 0);
    cx.simulate_keystrokes("right right");
    assert_eq!(selection(&editor, cx), 2..2);
    cx.simulate_keystrokes("shift-right");
    assert_eq!(selection(&editor, cx), 2..3);
    cx.simulate_keystrokes("left");
    assert_eq!(selection(&editor, cx), 2..2);
}

#[gpui::test]
fn control_arrows_move_by_word_and_home_end_by_line(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "abc def\nxyz");
    set_selection(&editor, cx, 0, 0);
    cx.simulate_keystrokes("ctrl-right");
    assert_eq!(selection(&editor, cx), 3..3);
    cx.simulate_keystrokes("end");
    assert_eq!(selection(&editor, cx), 7..7);
    cx.simulate_keystrokes("home");
    assert_eq!(selection(&editor, cx), 0..0);
    cx.simulate_keystrokes("ctrl-end");
    assert_eq!(selection(&editor, cx), 11..11);
    cx.simulate_keystrokes("ctrl-home");
    assert_eq!(selection(&editor, cx), 0..0);
}

#[gpui::test]
fn vertical_movement_keeps_the_column(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "abcdef\nxy\nabcdef");
    set_selection(&editor, cx, 4, 4);
    cx.simulate_keystrokes("down");
    assert_eq!(selection(&editor, cx), 9..9);
    cx.simulate_keystrokes("down");
    assert_eq!(selection(&editor, cx), 14..14);
    cx.simulate_keystrokes("up");
    assert_eq!(selection(&editor, cx), 9..9);
}

#[gpui::test]
fn select_all_covers_the_document(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "one\ntwo");
    cx.simulate_keystrokes("ctrl-a");
    assert_eq!(selection(&editor, cx), 0..7);
}

#[gpui::test]
fn backspace_and_delete_remove_characters(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "abcd");
    set_selection(&editor, cx, 2, 2);
    cx.simulate_keystrokes("backspace");
    assert_eq!(text(&editor, cx), "acd");
    cx.simulate_keystrokes("delete");
    assert_eq!(text(&editor, cx), "ad");
}

#[gpui::test]
fn control_backspace_and_delete_remove_words(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "one two three");
    set_selection(&editor, cx, 7, 7);
    cx.simulate_keystrokes("ctrl-backspace");
    assert_eq!(text(&editor, cx), "one  three");
    cx.simulate_keystrokes("ctrl-delete");
    assert_eq!(text(&editor, cx), "one ");
}

#[gpui::test]
fn enter_continues_a_list(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "- a");
    set_selection(&editor, cx, 3, 3);
    cx.simulate_keystrokes("enter");
    assert_eq!(text(&editor, cx), "- a\n- ");
}

#[gpui::test]
fn tab_indents_and_shift_tab_outdents_list_items(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "- a\n- b");
    set_selection(&editor, cx, 7, 7);
    cx.simulate_keystrokes("tab");
    assert_eq!(text(&editor, cx), "- a\n  - b");
    cx.simulate_keystrokes("shift-tab");
    assert_eq!(text(&editor, cx), "- a\n- b");
}

#[gpui::test]
fn tab_outside_a_list_inserts_spaces(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "word");
    set_selection(&editor, cx, 0, 0);
    cx.simulate_keystrokes("tab");
    assert_eq!(text(&editor, cx), "  word");
}

#[gpui::test]
fn undo_and_redo_restore_text(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "");
    cx.simulate_input("hello");
    cx.simulate_keystrokes("ctrl-z");
    assert_eq!(text(&editor, cx), "");
    cx.simulate_keystrokes("ctrl-shift-z");
    assert_eq!(text(&editor, cx), "hello");
    cx.simulate_keystrokes("ctrl-z ctrl-y");
    assert_eq!(text(&editor, cx), "hello");
}

#[gpui::test]
fn copy_and_paste_round_trip_through_the_clipboard(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "abc");
    cx.simulate_keystrokes("ctrl-a ctrl-c");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("abc")
    );
    cx.simulate_keystrokes("ctrl-end ctrl-v");
    assert_eq!(text(&editor, cx), "abcabc");
}

#[gpui::test]
fn cut_removes_the_selection_and_fills_the_clipboard(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "abcdef");
    set_selection(&editor, cx, 1, 4);
    cx.simulate_keystrokes("ctrl-x");
    assert_eq!(text(&editor, cx), "aef");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("bcd")
    );
}

#[gpui::test]
fn formatting_shortcuts_wrap_the_selection(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "word");
    set_selection(&editor, cx, 0, 4);
    cx.simulate_keystrokes("ctrl-b");
    assert_eq!(text(&editor, cx), "**word**");
    cx.simulate_keystrokes("ctrl-b");
    assert_eq!(text(&editor, cx), "word");
    cx.simulate_keystrokes("ctrl-i");
    assert_eq!(text(&editor, cx), "_word_");
}

#[gpui::test]
fn strikethrough_and_code_shortcuts_wrap_the_selection(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "word");
    set_selection(&editor, cx, 0, 4);
    cx.simulate_keystrokes("ctrl-shift-x");
    assert_eq!(text(&editor, cx), "~~word~~");
    cx.simulate_keystrokes("ctrl-shift-x");
    cx.simulate_keystrokes("ctrl-`");
    assert_eq!(text(&editor, cx), "`word`");
}

#[gpui::test]
fn control_enter_toggles_the_task_of_the_line(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "- [ ] a");
    set_selection(&editor, cx, 7, 7);
    cx.simulate_keystrokes("ctrl-enter");
    assert_eq!(text(&editor, cx), "- [x] a");
}

#[gpui::test]
fn changes_are_emitted_with_the_new_text(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "");
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    cx.update(|_, cx| {
        cx.subscribe(&editor, move |_, event: &EditorEvent, _| {
            if let EditorEvent::Changed(text) = event {
                sink.borrow_mut().push(text.clone());
            }
        })
        .detach();
    });
    cx.simulate_input("a");
    assert_eq!(seen.borrow().as_slice(), ["a"]);
}

#[gpui::test]
fn set_text_replaces_the_document_without_an_event(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "old");
    let seen = Rc::new(RefCell::new(0));
    let sink = seen.clone();
    cx.update(|_, cx| {
        cx.subscribe(&editor, move |_, _: &EditorEvent, _| {
            *sink.borrow_mut() += 1;
        })
        .detach();
    });
    editor.update(cx, |editor, cx| editor.set_text("brand new", cx));
    assert_eq!(text(&editor, cx), "brand new");
    assert_eq!(*seen.borrow(), 0);
}

#[gpui::test]
fn clicking_places_the_caret(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "hello world");
    cx.simulate_click(point(px(column_x(3.0)), px(row_y(0.0))), Modifiers::none());
    assert_eq!(selection(&editor, cx), 3..3);
}

#[gpui::test]
fn shift_click_extends_the_selection(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "hello world");
    set_selection(&editor, cx, 2, 2);
    cx.simulate_click(point(px(column_x(7.0)), px(row_y(0.0))), Modifiers::shift());
    assert_eq!(selection(&editor, cx), 2..7);
}

#[gpui::test]
fn dragging_selects_a_range(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "hello world");
    cx.simulate_mouse_down(
        point(px(column_x(1.0)), px(row_y(0.0))),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_move(
        point(px(column_x(5.0)), px(row_y(0.0))),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_up(
        point(px(column_x(5.0)), px(row_y(0.0))),
        MouseButton::Left,
        Modifiers::none(),
    );
    assert_eq!(selection(&editor, cx), 1..5);
}

fn click_with_count(cx: &mut VisualTestContext, x: f32, y: f32, count: usize) {
    let position = point(px(x), px(y));
    cx.simulate_event(MouseDownEvent {
        position,
        modifiers: Modifiers::none(),
        button: MouseButton::Left,
        click_count: count,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position,
        modifiers: Modifiers::none(),
        button: MouseButton::Left,
        click_count: count,
    });
}

#[gpui::test]
fn double_click_selects_a_word_and_triple_click_a_line(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "hello world\nnext");
    click_with_count(cx, column_x(7.0), row_y(0.0), 2);
    assert_eq!(selection(&editor, cx), 6..11);
    click_with_count(cx, column_x(7.0), row_y(0.0), 3);
    assert_eq!(selection(&editor, cx), 0..12);
}

#[gpui::test]
fn clicking_a_task_marker_toggles_it_without_moving_the_caret(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "- [ ] task\nother");
    set_selection(&editor, cx, 13, 13);
    let toggled = Rc::new(RefCell::new(Vec::new()));
    let sink = toggled.clone();
    cx.update(|_, cx| {
        cx.subscribe(&editor, move |_, event: &EditorEvent, _| {
            if let EditorEvent::TaskToggled { line } = event {
                sink.borrow_mut().push(*line);
            }
        })
        .detach();
    });
    cx.simulate_click(point(px(column_x(3.0)), px(row_y(0.0))), Modifiers::none());
    assert_eq!(text(&editor, cx), "- [x] task\nother");
    assert_eq!(selection(&editor, cx), 13..13);
    assert_eq!(toggled.borrow().as_slice(), [0]);
}

#[gpui::test]
fn line_numbers_shift_the_text_and_are_off_by_default(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "hello");
    assert!(!editor.read_with(cx, |editor, _| editor.line_numbers()));
    editor.update(cx, |editor, cx| editor.set_line_numbers(true, cx));
    cx.run_until_parked();
    cx.simulate_click(point(px(column_x(3.0)), px(row_y(0.0))), Modifiers::none());
    assert_eq!(selection(&editor, cx), 0..0);
}

fn long_document() -> String {
    "line\n".repeat(200)
}

#[gpui::test]
fn the_mouse_wheel_scrolls_vertically(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, &long_document());
    assert_eq!(
        editor.read_with(cx, |editor, _| editor.scroll_top()),
        px(0.)
    );
    cx.simulate_event(ScrollEvent {
        position: point(px(100.), px(100.)),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        modifiers: Modifiers::none(),
        touch_phase: gpui::TouchPhase::Moved,
        source: ScrollSource::Pointer,
    });
    cx.run_until_parked();
    assert_eq!(
        editor.read_with(cx, |editor, _| editor.scroll_top()),
        px(120.)
    );
}

#[gpui::test]
fn scrolling_stops_at_the_top(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, &long_document());
    cx.simulate_event(ScrollEvent {
        position: point(px(100.), px(100.)),
        delta: ScrollDelta::Pixels(point(px(0.), px(300.))),
        modifiers: Modifiers::none(),
        touch_phase: gpui::TouchPhase::Moved,
        source: ScrollSource::Pointer,
    });
    assert_eq!(
        editor.read_with(cx, |editor, _| editor.scroll_top()),
        px(0.)
    );
}

#[gpui::test]
fn moving_the_caret_far_down_scrolls_it_into_view(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, &long_document());
    cx.simulate_keystrokes("ctrl-end");
    cx.run_until_parked();
    assert!(editor.read_with(cx, |editor, _| editor.scroll_top()) > px(1000.));
}

#[gpui::test]
fn a_long_line_wraps_and_down_moves_within_it(cx: &mut TestAppContext) {
    let words = "word ".repeat(400);
    let (editor, cx) = open(cx, &words);
    set_selection(&editor, cx, 0, 0);
    cx.simulate_keystrokes("down");
    assert!(selection(&editor, cx).start > 0);
    assert!(selection(&editor, cx).start < words.len());
}

#[gpui::test]
fn toggling_a_task_by_line_leaves_the_caret_alone(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "- [ ] a\n- [ ] b");
    set_selection(&editor, cx, 15, 15);
    editor.update(cx, |editor, cx| editor.toggle_task_at_line(1, cx));
    assert_eq!(text(&editor, cx), "- [ ] a\n- [x] b");
    assert_eq!(selection(&editor, cx), 15..15);
}

#[gpui::test]
fn toggling_a_task_past_the_last_line_is_ignored(cx: &mut TestAppContext) {
    let (editor, cx) = open(cx, "- [ ] a");
    editor.update(cx, |editor, cx| editor.toggle_task_at_line(9, cx));
    assert_eq!(text(&editor, cx), "- [ ] a");
}
