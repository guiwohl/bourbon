use std::cell::RefCell;
use std::rc::Rc;

use bourbon_editor::{TextField, TextFieldEvent, bind_text_field_keys};
use bourbon_ui::{Appearance, IconName, Theme};
use gpui::{
    AppContext as _, ClipboardItem, Context, Entity, EntityInputHandler, IntoElement, Modifiers,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Render,
    Styled as _, TestAppContext, VisualTestContext, Window, div, point, px,
};

const CHAR_WIDTH: f32 = 8.4;
const TEXT_LEFT: f32 = 9.0;
const ICON_SHIFT: f32 = 24.0;
const FIELD_MIDDLE: f32 = 15.0;
const FIELD_WIDTH: f32 = 200.0;

struct Host {
    field: Entity<TextField>,
}

impl Render for Host {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(FIELD_WIDTH)).child(self.field.clone())
    }
}

type Events = Rc<RefCell<Vec<TextFieldEvent>>>;

struct Opened<'a> {
    field: Entity<TextField>,
    events: Events,
    cx: &'a mut VisualTestContext,
}

fn open_with<'a>(
    cx: &'a mut TestAppContext,
    text: &str,
    build: impl FnOnce(TextField) -> TextField + 'static,
) -> Opened<'a> {
    cx.update(|cx| {
        Theme::install(Appearance::Light, cx);
        bind_text_field_keys(cx);
    });
    let field = cx.new(|cx| build(TextField::new(cx).placeholder("Filter")));
    let events: Events = Rc::default();
    let recorded = events.clone();
    cx.update(|cx| {
        cx.subscribe(&field, move |_, event: &TextFieldEvent, _| {
            recorded.borrow_mut().push(event.clone());
        })
        .detach();
    });
    let hosted = field.clone();
    let (_, cx) = cx.add_window_view(move |_, _| Host { field: hosted });
    let text = text.to_owned();
    field.update_in(cx, |field, window, cx| {
        field.set_text(&text, cx);
        field.focus(window, cx);
    });
    cx.run_until_parked();
    events.borrow_mut().clear();
    Opened { field, events, cx }
}

fn open<'a>(cx: &'a mut TestAppContext, text: &str) -> Opened<'a> {
    open_with(cx, text, |field| field)
}

impl Opened<'_> {
    fn text(&mut self) -> String {
        self.field.read_with(self.cx, |field, _| field.text())
    }

    fn selection(&mut self) -> std::ops::Range<usize> {
        self.field
            .read_with(self.cx, |field, _| field.selected_range())
    }

    fn keys(&mut self, keystrokes: &str) {
        self.cx.simulate_keystrokes(keystrokes);
    }

    fn events(&self) -> Vec<TextFieldEvent> {
        self.events.borrow().clone()
    }

    fn mouse_down(&mut self, x: f32, click_count: usize, shift: bool) {
        self.cx.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position: point(px(x), px(FIELD_MIDDLE)),
            modifiers: Modifiers {
                shift,
                ..Modifiers::default()
            },
            click_count,
            first_mouse: false,
        });
    }

    fn mouse_up(&mut self, x: f32) {
        self.cx.simulate_event(MouseUpEvent {
            button: MouseButton::Left,
            position: point(px(x), px(FIELD_MIDDLE)),
            modifiers: Modifiers::default(),
            click_count: 1,
        });
    }

    fn drag_to(&mut self, x: f32) {
        self.cx.simulate_event(MouseMoveEvent {
            position: point(px(x), px(FIELD_MIDDLE)),
            pressed_button: Some(MouseButton::Left),
            modifiers: Modifiers::default(),
        });
    }
}

fn column_x(column: f32) -> f32 {
    TEXT_LEFT + column * CHAR_WIDTH + 1.0
}

#[gpui::test]
fn typing_inserts_text_and_reports_each_change(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    field.cx.simulate_input("ab");
    assert_eq!(field.text(), "ab");
    assert_eq!(field.selection(), 2..2);
    assert_eq!(
        field.events(),
        [
            TextFieldEvent::Changed("a".to_owned()),
            TextFieldEvent::Changed("ab".to_owned())
        ]
    );
}

#[gpui::test]
fn typing_a_bracket_inserts_it_alone(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    field.cx.simulate_input("(");
    assert_eq!(field.text(), "(");
}

#[gpui::test]
fn enter_submits_without_inserting_a_line_break(cx: &mut TestAppContext) {
    let mut field = open(cx, "name");
    field.keys("enter");
    assert_eq!(field.text(), "name");
    assert_eq!(field.events(), [TextFieldEvent::Submit]);
}

#[gpui::test]
fn escape_cancels_and_keeps_the_text(cx: &mut TestAppContext) {
    let mut field = open(cx, "name");
    field.keys("escape");
    assert_eq!(field.text(), "name");
    assert_eq!(field.events(), [TextFieldEvent::Cancel]);
}

#[gpui::test]
fn set_text_drops_line_breaks_puts_the_caret_at_the_end_and_stays_silent(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    field
        .field
        .update(field.cx, |field, cx| field.set_text("a\nb", cx));
    assert_eq!(field.text(), "ab");
    assert_eq!(field.selection(), 2..2);
    assert_eq!(field.events(), []);
    field.keys("ctrl-z");
    assert_eq!(field.text(), "ab");
}

#[gpui::test]
fn backspace_and_delete_remove_one_character_each(cx: &mut TestAppContext) {
    let mut field = open(cx, "abcd");
    field.keys("left left backspace delete");
    assert_eq!(field.text(), "ad");
    assert_eq!(field.selection(), 1..1);
}

#[gpui::test]
fn word_deletes_remove_whole_words(cx: &mut TestAppContext) {
    let mut field = open(cx, "one two three");
    field.keys("ctrl-backspace");
    assert_eq!(field.text(), "one two ");
    field.keys("home ctrl-delete");
    assert_eq!(field.text(), " two ");
}

#[gpui::test]
fn home_end_and_word_moves_walk_the_line(cx: &mut TestAppContext) {
    let mut field = open(cx, "one two");
    field.keys("home");
    assert_eq!(field.selection(), 0..0);
    field.keys("ctrl-right");
    assert_eq!(field.selection(), 3..3);
    field.keys("right");
    assert_eq!(field.selection(), 4..4);
    field.keys("end");
    assert_eq!(field.selection(), 7..7);
    field.keys("ctrl-left");
    assert_eq!(field.selection(), 4..4);
    field.keys("left");
    assert_eq!(field.selection(), 3..3);
}

#[gpui::test]
fn shifted_moves_extend_the_selection(cx: &mut TestAppContext) {
    let mut field = open(cx, "one two");
    field.keys("shift-left");
    assert_eq!(field.selection(), 6..7);
    field.keys("shift-right");
    assert_eq!(field.selection(), 7..7);
    field.keys("ctrl-shift-left");
    assert_eq!(field.selection(), 4..7);
    field.keys("shift-home");
    assert_eq!(field.selection(), 0..7);
    field.keys("home ctrl-shift-right");
    assert_eq!(field.selection(), 0..3);
    field.keys("shift-end");
    assert_eq!(field.selection(), 0..7);
}

#[gpui::test]
fn select_all_then_typing_replaces_everything(cx: &mut TestAppContext) {
    let mut field = open(cx, "old name");
    field.keys("ctrl-a");
    assert_eq!(field.selection(), 0..8);
    field.cx.simulate_input("new");
    assert_eq!(field.text(), "new");
}

#[gpui::test]
fn undo_and_redo_walk_the_history(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    field.cx.simulate_input("ab");
    field.keys("ctrl-z");
    assert_eq!(field.text(), "");
    field.keys("ctrl-shift-z");
    assert_eq!(field.text(), "ab");
    field.keys("ctrl-z ctrl-y");
    assert_eq!(field.text(), "ab");
    assert_eq!(
        field.events().last(),
        Some(&TextFieldEvent::Changed("ab".to_owned()))
    );
}

#[gpui::test]
fn copy_and_cut_put_the_selection_on_the_clipboard(cx: &mut TestAppContext) {
    let mut field = open(cx, "one two");
    field.keys("ctrl-c");
    assert_eq!(field.cx.read_from_clipboard(), None);
    field.keys("ctrl-shift-left ctrl-c");
    assert_eq!(
        field.cx.read_from_clipboard().and_then(|item| item.text()),
        Some("two".to_owned())
    );
    assert_eq!(field.text(), "one two");
    field.keys("home ctrl-shift-right ctrl-x");
    assert_eq!(
        field.cx.read_from_clipboard().and_then(|item| item.text()),
        Some("one".to_owned())
    );
    assert_eq!(field.text(), " two");
}

#[gpui::test]
fn cut_without_a_selection_keeps_the_text(cx: &mut TestAppContext) {
    let mut field = open(cx, "one");
    field.keys("ctrl-x");
    assert_eq!(field.text(), "one");
}

#[gpui::test]
fn paste_strips_line_breaks_and_undoes_in_one_step(cx: &mut TestAppContext) {
    let mut field = open(cx, "x");
    field
        .cx
        .write_to_clipboard(ClipboardItem::new_string("a\r\nb\nc".to_owned()));
    field.keys("ctrl-v");
    assert_eq!(field.text(), "xabc");
    field.keys("ctrl-z");
    assert_eq!(field.text(), "x");
}

#[gpui::test]
fn ime_composition_marks_then_commits(cx: &mut TestAppContext) {
    let mut field = open(cx, "ab");
    field.keys("left");
    field.field.update_in(field.cx, |field, window, cx| {
        field.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
    });
    assert_eq!(field.text(), "anib");
    let marked = field.field.update_in(field.cx, |field, window, cx| {
        field.marked_text_range(window, cx)
    });
    assert_eq!(marked, Some(1..3));
    assert_eq!(field.selection(), 3..3);
    field.field.update_in(field.cx, |field, window, cx| {
        field.replace_text_in_range(None, "你", window, cx);
    });
    assert_eq!(field.text(), "a你b");
    assert_eq!(field.selection(), 4..4);
    let marked = field.field.update_in(field.cx, |field, window, cx| {
        field.marked_text_range(window, cx)
    });
    assert_eq!(marked, None);
}

#[gpui::test]
fn committed_input_never_holds_a_line_break(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    field.field.update_in(field.cx, |field, window, cx| {
        field.replace_text_in_range(None, "a\nb", window, cx);
    });
    assert_eq!(field.text(), "ab");
}

#[gpui::test]
fn the_input_handler_speaks_utf16(cx: &mut TestAppContext) {
    let mut field = open(cx, "a🙂b");
    let (selected, length, slice) = field.field.update_in(field.cx, |field, window, cx| {
        let selected = field
            .selected_text_range(false, window, cx)
            .map(|selection| selection.range);
        let length = field.text_length_utf16(window, cx);
        let mut adjusted = None;
        let slice = field.text_for_range(1..3, &mut adjusted, window, cx);
        (selected, length, slice)
    });
    assert_eq!(selected, Some(4..4));
    assert_eq!(length, Some(4));
    assert_eq!(slice, Some("🙂".to_owned()));
    field.field.update_in(field.cx, |field, window, cx| {
        field.set_selected_text_range(1..3, window, cx);
    });
    assert_eq!(field.selection(), 1..5);
}

#[gpui::test]
fn ime_bounds_and_hit_tests_follow_the_text(cx: &mut TestAppContext) {
    let field = open(cx, "abcd");
    let (bounds, index) = field.field.update_in(field.cx, |field, window, cx| {
        let element = gpui::Bounds::default();
        (
            field.bounds_for_range(1..3, element, window, cx),
            field.character_index_for_point(point(px(column_x(2.0)), px(FIELD_MIDDLE)), window, cx),
        )
    });
    let bounds = bounds.expect("bounds");
    assert!((f32::from(bounds.origin.x) - (TEXT_LEFT + CHAR_WIDTH)).abs() < 0.05);
    assert!((f32::from(bounds.size.width) - 2.0 * CHAR_WIDTH).abs() < 0.05);
    assert_eq!(index, Some(2));
}

#[gpui::test]
fn clicking_places_the_caret_under_the_pointer(cx: &mut TestAppContext) {
    let mut field = open(cx, "abcdef");
    field.mouse_down(column_x(2.0), 1, false);
    field.mouse_up(column_x(2.0));
    assert_eq!(field.selection(), 2..2);
    field.mouse_down(column_x(4.0), 1, true);
    assert_eq!(field.selection(), 2..4);
}

#[gpui::test]
fn the_leading_icon_pushes_the_text_right(cx: &mut TestAppContext) {
    let mut field = open_with(cx, "abcdef", |field| field.leading_icon(IconName::Search));
    field.mouse_down(column_x(4.0), 1, false);
    assert_eq!(field.selection(), 1..1);
    field.mouse_down(column_x(4.0) + ICON_SHIFT, 1, false);
    assert_eq!(field.selection(), 4..4);
}

#[gpui::test]
fn dragging_selects_until_the_pointer_and_stops_on_release(cx: &mut TestAppContext) {
    let mut field = open(cx, "abcdef");
    field.mouse_down(column_x(1.0), 1, false);
    field.drag_to(column_x(4.0));
    assert_eq!(field.selection(), 1..4);
    field.mouse_up(column_x(4.0));
    field.drag_to(column_x(5.0));
    assert_eq!(field.selection(), 1..4);
}

#[gpui::test]
fn double_click_selects_a_word_and_triple_click_everything(cx: &mut TestAppContext) {
    let mut field = open(cx, "one two");
    field.mouse_down(column_x(5.0), 2, false);
    assert_eq!(field.selection(), 4..7);
    field.mouse_down(column_x(1.0), 3, false);
    assert_eq!(field.selection(), 0..7);
}

#[gpui::test]
fn clicking_the_placeholder_keeps_the_caret_at_the_start(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    field.mouse_down(column_x(4.0), 1, false);
    assert_eq!(field.selection(), 0..0);
    assert_eq!(field.text(), "");
}

#[gpui::test]
fn a_long_text_scrolls_to_keep_the_caret_visible(cx: &mut TestAppContext) {
    let mut field = open(cx, &"x".repeat(60));
    field.cx.run_until_parked();
    let caret_x = |field: &mut Opened<'_>| {
        field.field.update_in(field.cx, |field, window, cx| {
            let caret = field
                .selected_text_range(false, window, cx)
                .expect("selection")
                .range
                .start;
            f32::from(
                field
                    .bounds_for_range(caret..caret, gpui::Bounds::default(), window, cx)
                    .expect("bounds")
                    .origin
                    .x,
            )
        })
    };
    let at_end = caret_x(&mut field);
    assert!(at_end <= FIELD_WIDTH - TEXT_LEFT);
    assert!(at_end > FIELD_WIDTH - TEXT_LEFT - CHAR_WIDTH);
    field.keys("home");
    assert!((caret_x(&mut field) - TEXT_LEFT).abs() < 0.05);
}

#[gpui::test]
fn the_composition_selection_lands_inside_the_marked_text(cx: &mut TestAppContext) {
    let mut field = open(cx, "abc");
    field.keys("left left");
    field.field.update_in(field.cx, |field, window, cx| {
        field.replace_and_mark_text_in_range(None, "xyz", Some(1..2), window, cx);
    });
    assert_eq!(field.text(), "axyzbc");
    assert_eq!(field.selection(), 2..3);
}

#[gpui::test]
fn unmarking_ends_the_composition_and_keeps_its_text(cx: &mut TestAppContext) {
    let mut field = open(cx, "");
    let marked = field.field.update_in(field.cx, |field, window, cx| {
        field.replace_and_mark_text_in_range(None, "ni", None, window, cx);
        field.unmark_text(window, cx);
        field.marked_text_range(window, cx)
    });
    assert_eq!(marked, None);
    assert_eq!(field.text(), "ni");
}

#[gpui::test]
fn clicking_a_scrolled_field_counts_the_hidden_text(cx: &mut TestAppContext) {
    let mut field = open(cx, &"x".repeat(60));
    field.cx.run_until_parked();
    field.mouse_down(column_x(0.0), 1, false);
    let offset = field.selection().start;
    assert!((38..=39).contains(&offset), "offset {offset}");
}
