mod element;
mod geometry;
mod input;
mod keys;
mod mask;
mod mouse;
mod view;

use std::ops::Range;
use std::time::Instant;

use bourbon_text::commands::{delete_backward, delete_forward, delete_word_backward};
use bourbon_text::{
    EditorState, History, Selection, SelectionRange, Text, Transaction, TransactionKind,
};
use bourbon_ui::IconName;
use gpui::{
    App, ClipboardItem, Context, EventEmitter, FocusHandle, Focusable, IntoElement, Pixels, Render,
    SharedString, Window,
};

use crate::blink::schedule_blink;
use crate::commands::{delete_next_word, paste_text};
use crate::motion::{Motion, move_range};

pub use keys::{TEXT_FIELD_CONTEXT, bind_text_field_keys};

use geometry::LineGeometry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextFieldEvent {
    Changed(String),
    Submit,
    Cancel,
}

#[derive(Clone, Copy, Debug, Default)]
struct Look {
    masked: bool,
    borderless: bool,
    text_size: Option<Pixels>,
}

pub struct TextField {
    state: EditorState,
    history: History,
    focus_handle: FocusHandle,
    placeholder: SharedString,
    leading_icon: Option<IconName>,
    marked: Option<Range<usize>>,
    caret_visible: bool,
    blink_epoch: u64,
    started: Instant,
    dragging: bool,
    geometry: LineGeometry,
    look: Look,
}

impl EventEmitter<TextFieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub(crate) fn single_line(text: &str) -> String {
    text.chars()
        .filter(|character| !matches!(character, '\n' | '\r'))
        .collect()
}

impl TextField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut field = Self {
            state: EditorState::new(Text::new(""), Selection::cursor(0)),
            history: History::new(),
            focus_handle: cx.focus_handle(),
            placeholder: SharedString::default(),
            leading_icon: None,
            marked: None,
            caret_visible: true,
            blink_epoch: 0,
            started: Instant::now(),
            dragging: false,
            geometry: LineGeometry::default(),
            look: Look::default(),
        };
        field.restart_blink(cx);
        field
    }

    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    #[must_use]
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    #[must_use]
    pub fn masked(mut self, masked: bool) -> Self {
        self.look.masked = masked;
        self
    }

    #[must_use]
    pub fn borderless(mut self) -> Self {
        self.look.borderless = true;
        self
    }

    #[must_use]
    pub fn text_size(mut self, size: Pixels) -> Self {
        self.look.text_size = Some(size);
        self
    }

    pub fn text(&self) -> String {
        self.state.text.to_string()
    }

    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        let text = single_line(text);
        self.state = EditorState::new(Text::new(&text), Selection::cursor(text.len()));
        self.history = History::new();
        self.marked = None;
        self.caret_moved(cx);
    }

    pub fn selected_range(&self) -> Range<usize> {
        let main = self.state.selection.main();
        main.from()..main.to()
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    fn select(&mut self, range: SelectionRange, cx: &mut Context<Self>) {
        self.state.selection = Selection::single(range);
        self.marked = None;
        self.caret_moved(cx);
    }

    fn caret_moved(&mut self, cx: &mut Context<Self>) {
        self.restart_blink(cx);
        cx.notify();
    }

    fn run_command(
        &mut self,
        command: impl FnOnce(&EditorState) -> Option<Transaction>,
        cx: &mut Context<Self>,
    ) {
        if let Some(transaction) = command(&self.state) {
            self.apply(&transaction, cx);
        }
    }

    fn apply(&mut self, transaction: &Transaction, cx: &mut Context<Self>) {
        let recorded = !matches!(
            transaction.kind,
            TransactionKind::Undo | TransactionKind::Redo
        );
        if recorded {
            self.history
                .record(&self.state, transaction, self.started.elapsed());
        }
        self.state = self.state.apply(transaction);
        self.marked = None;
        self.caret_moved(cx);
        if !transaction.changes.is_empty() {
            cx.emit(TextFieldEvent::Changed(self.text()));
        }
    }

    fn move_caret(&mut self, motion: Motion, extend: bool, cx: &mut Context<Self>) {
        let moved = move_range(
            &self.state.text,
            self.state.selection.main(),
            motion,
            extend,
        );
        self.select(moved, cx);
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        self.select(SelectionRange::new(0, self.state.text.len()), cx);
    }

    fn backspace(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_backward, cx);
    }

    fn delete(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_forward, cx);
    }

    fn delete_word_backward(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_word_backward, cx);
    }

    fn delete_word_forward(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_next_word, cx);
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some(transaction) = self.history.undo(&self.state) {
            self.apply(&transaction, cx);
        }
    }

    fn redo(&mut self, cx: &mut Context<Self>) {
        if let Some(transaction) = self.history.redo(&self.state) {
            self.apply(&transaction, cx);
        }
    }

    fn copy(&mut self, cx: &mut Context<Self>) -> bool {
        let selected = self.state.selection.main();
        if selected.is_empty() {
            return false;
        }
        let text = self.state.text.slice(selected.from()..selected.to());
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        true
    }

    fn cut(&mut self, cx: &mut Context<Self>) {
        if self.copy(cx) {
            self.run_command(delete_backward, cx);
        }
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let text = single_line(&text);
        self.run_command(|state| paste_text(state, &text), cx);
    }

    fn restart_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_visible = true;
        self.blink_epoch += 1;
        schedule_blink(self.blink_epoch, cx, Self::blink);
    }

    fn blink(&mut self, epoch: u64, cx: &mut Context<Self>) {
        if epoch != self.blink_epoch {
            return;
        }
        self.caret_visible = !self.caret_visible;
        cx.notify();
        schedule_blink(epoch, cx, Self::blink);
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        view::render(self, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blink::BLINK_INTERVAL;
    use gpui::{AppContext as _, TestAppContext};

    #[gpui::test]
    fn the_caret_blinks_and_any_edit_shows_it_again(cx: &mut TestAppContext) {
        let field = cx.new(TextField::new);
        let visible = |cx: &mut TestAppContext| field.read_with(cx, |field, _| field.caret_visible);
        assert!(visible(cx));
        cx.executor().advance_clock(BLINK_INTERVAL);
        assert!(!visible(cx));
        cx.executor().advance_clock(BLINK_INTERVAL);
        assert!(visible(cx));
        cx.executor().advance_clock(BLINK_INTERVAL);
        field.update(cx, |field, cx| field.set_text("a", cx));
        assert!(visible(cx));
        cx.executor().advance_clock(BLINK_INTERVAL / 2);
        assert!(visible(cx));
        cx.executor().advance_clock(BLINK_INTERVAL / 2);
        assert!(!visible(cx));
    }

    #[test]
    fn single_line_strips_every_line_break() {
        assert_eq!(single_line("a\nb\r\nc\rd"), "abcd");
        assert_eq!(single_line("plain"), "plain");
    }
}
