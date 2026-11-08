use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};
use std::rc::Rc;
use std::time::Instant;

use bourbon_markdown::{HighlightSpan, LineDecoration, fold_ranges, highlight, line_decorations};
use bourbon_text::{
    EditorState, Folds, History, Selection, SelectionRange, Text, Transaction, TransactionKind,
};
use gpui::{
    Bounds, Context, EventEmitter, FocusHandle, Focusable, IntoElement, Pixels, Render, Window, px,
};

use crate::blink::schedule_blink;
use crate::layout::{LineLayout, ShapeRequest};
use crate::line_decor::{LineDecor, decor_of, line_box};
use crate::line_heights::LineHeights;
use crate::segments::{heading_level, inline_code_ranges, line_segments};
use crate::theme::EditorTheme;

const LAYOUT_CACHE_LIMIT: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorEvent {
    Changed(String),
    TaskToggled { line: usize },
    Scrolled,
}

#[allow(clippy::struct_excessive_bools)]
pub struct Editor {
    pub(crate) state: EditorState,
    pub(crate) history: History,
    pub(crate) spans: Vec<HighlightSpan>,
    decorations: Vec<LineDecoration>,
    pub(crate) fold_ranges: Vec<RangeInclusive<usize>>,
    pub(crate) folds: Folds,
    pub(crate) theme: Rc<EditorTheme>,
    pub(crate) heights: LineHeights,
    pub(crate) scroll_top: Pixels,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) caret_visible: bool,
    pub(crate) focused: bool,
    blink_epoch: u64,
    pub(crate) marked: Option<Range<usize>>,
    pub(crate) goal_x: Option<Pixels>,
    pub(crate) line_numbers: bool,
    started: Instant,
    layouts: HashMap<usize, Rc<LineLayout>>,
    pub(crate) viewport: Option<Bounds<Pixels>>,
    pub(crate) gutter_width: Pixels,
    pub(crate) text_left: Pixels,
    pub(crate) wrap_width: Option<Pixels>,
    pub(crate) dragging: bool,
    pub(crate) scroll_to_caret: bool,
    reported_scroll: Pixels,
    pub(crate) search_matches: Vec<Range<usize>>,
}

impl EventEmitter<EditorEvent> for Editor {}

impl Focusable for Editor {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Editor {
    pub fn new(text: &str, theme: EditorTheme, cx: &mut Context<Self>) -> Self {
        let state = EditorState::new(Text::new(text), Selection::cursor(0));
        let estimate = line_height_of(&theme, theme.font_size);
        let mut editor = Self {
            spans: highlight(text),
            decorations: line_decorations(text),
            fold_ranges: fold_ranges(text),
            folds: Folds::default(),
            heights: LineHeights::new(state.text.line_count(), f32::from(estimate)),
            state,
            history: History::new(),
            theme: Rc::new(theme),
            scroll_top: px(0.0),
            focus_handle: cx.focus_handle(),
            caret_visible: true,
            focused: false,
            blink_epoch: 0,
            marked: None,
            goal_x: None,
            line_numbers: false,
            started: Instant::now(),
            layouts: HashMap::new(),
            viewport: None,
            gutter_width: px(0.0),
            text_left: px(0.0),
            wrap_width: None,
            dragging: false,
            scroll_to_caret: false,
            reported_scroll: px(0.0),
            search_matches: Vec::new(),
        };
        editor.restart_blink(cx);
        editor
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn text(&self) -> String {
        self.state.text.to_string()
    }

    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.state = EditorState::new(Text::new(text), Selection::cursor(0));
        self.history = History::new();
        self.folds.clear();
        self.marked = None;
        self.scroll_top = px(0.0);
        self.refresh_after_text_change(cx);
        cx.notify();
    }

    pub fn theme(&self) -> &EditorTheme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: EditorTheme, cx: &mut Context<Self>) {
        self.theme = Rc::new(theme);
        self.invalidate_all_heights();
        cx.notify();
    }

    pub fn line_numbers(&self) -> bool {
        self.line_numbers
    }

    pub fn set_line_numbers(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.line_numbers = enabled;
        self.invalidate_all_heights();
        cx.notify();
    }

    pub fn scroll_top(&self) -> Pixels {
        self.scroll_top
    }

    pub fn top_line(&self) -> f32 {
        self.heights.line_position(f32::from(self.scroll_top))
    }

    pub fn scroll_to_line(&mut self, line: f32, cx: &mut Context<Self>) {
        self.scroll_top = px(self.heights.offset_of_line(line))
            .min(self.max_scroll())
            .max(px(0.0));
        cx.notify();
    }

    pub(crate) fn report_scroll(&mut self, cx: &mut Context<Self>) {
        if self.scroll_top != self.reported_scroll {
            self.reported_scroll = self.scroll_top;
            cx.emit(EditorEvent::Scrolled);
        }
    }

    pub fn selected_range(&self) -> Range<usize> {
        let main = self.state.selection.main();
        main.from()..main.to()
    }

    pub fn set_selection(&mut self, anchor: usize, head: usize, cx: &mut Context<Self>) {
        let length = self.state.text.len();
        let (anchor, head) = (anchor.min(length), head.min(length));
        self.reveal(anchor);
        self.reveal(head);
        self.select(SelectionRange::new(anchor, head), cx);
    }

    pub(crate) fn select(&mut self, range: SelectionRange, cx: &mut Context<Self>) {
        self.set_visible_selection(range);
        self.marked = None;
        self.goal_x = None;
        self.caret_moved(cx);
    }

    pub(crate) fn select_keeping_goal(&mut self, range: SelectionRange, cx: &mut Context<Self>) {
        self.set_visible_selection(range);
        self.marked = None;
        self.caret_moved(cx);
    }

    fn set_visible_selection(&mut self, range: SelectionRange) {
        let previous_head = self.state.selection.main().head;
        self.state.selection = Selection::single(self.visible_selection(range, previous_head));
    }

    fn caret_moved(&mut self, cx: &mut Context<Self>) {
        self.scroll_to_caret = true;
        self.restart_blink(cx);
        cx.notify();
    }

    pub(crate) fn run_command(
        &mut self,
        command: impl FnOnce(&EditorState) -> Option<Transaction>,
        cx: &mut Context<Self>,
    ) -> bool {
        match command(&self.state) {
            Some(transaction) => {
                self.apply(&transaction, cx);
                true
            }
            None => false,
        }
    }

    pub(crate) fn apply(&mut self, transaction: &Transaction, cx: &mut Context<Self>) {
        let recorded = !matches!(
            transaction.kind,
            TransactionKind::Undo | TransactionKind::Redo
        );
        if recorded {
            self.history
                .record(&self.state, transaction, self.started.elapsed());
        }
        let before = self.state.clone();
        self.state = before.apply(transaction);
        self.folds = self.folds.map(&transaction.changes);
        if self.folds.hides(self.state.selection.main().head) {
            self.set_visible_selection(self.state.selection.main());
        }
        self.marked = None;
        self.goal_x = None;
        if transaction.changes.is_empty() {
            self.caret_moved(cx);
            return;
        }
        self.heights
            .apply_changes(&before.text, &transaction.changes);
        self.refresh_after_text_change(cx);
        self.scroll_to_caret = true;
        self.restart_blink(cx);
        cx.emit(EditorEvent::Changed(self.text()));
        cx.notify();
    }

    pub(crate) fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some(transaction) = self.history.undo(&self.state) {
            self.apply(&transaction, cx);
        }
    }

    pub(crate) fn redo(&mut self, cx: &mut Context<Self>) {
        if let Some(transaction) = self.history.redo(&self.state) {
            self.apply(&transaction, cx);
        }
    }

    fn refresh_after_text_change(&mut self, _cx: &mut Context<Self>) {
        let source = self.state.text.to_string();
        self.spans = highlight(&source);
        self.decorations = line_decorations(&source);
        self.fold_ranges = fold_ranges(&source);
        if self.heights.line_count() != self.state.text.line_count() {
            let estimate = line_height_of(&self.theme, self.theme.font_size);
            self.heights = LineHeights::new(self.state.text.line_count(), f32::from(estimate));
        }
        self.heights.refresh();
        self.layouts.clear();
    }

    pub(crate) fn invalidate_all_heights(&mut self) {
        let estimate = line_height_of(&self.theme, self.theme.font_size);
        self.heights.reset(f32::from(estimate));
        self.heights.refresh();
        self.layouts.clear();
        self.wrap_width = None;
    }

    pub(crate) fn invalidate_layouts(&mut self) {
        self.layouts.clear();
    }

    pub(crate) fn clear_layouts(&mut self) {
        self.layouts.clear();
    }

    pub(crate) fn restart_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_visible = true;
        self.blink_epoch += 1;
        schedule_blink(self.blink_epoch, cx, Self::blink);
    }

    fn blink(&mut self, epoch: u64, cx: &mut Context<Self>) {
        if epoch != self.blink_epoch {
            return;
        }
        self.caret_visible = !self.caret_visible;
        if self.focused {
            cx.notify();
        }
        schedule_blink(epoch, cx, Self::blink);
    }

    pub(crate) fn shape_line(&self, line: usize, window: &mut Window) -> LineLayout {
        let text = &self.state.text;
        let start = text.line_start(line);
        let end = text.line_end(line);
        let content = text.line_text(line);
        let level = heading_level(start..end, &self.spans);
        let scale = level.map_or(1.0, |level| self.theme.heading_scale(level));
        let font_size = self.theme.font_size * scale;
        let segments = line_segments(start..end, &self.spans, self.marked.as_ref());
        let box_model = line_box(self.line_decor(line), &self.theme);
        LineLayout::shape(
            &ShapeRequest {
                text: &content,
                segments: &segments,
                font_size,
                line_height: line_height_of(&self.theme, font_size),
                tall_ranges: &inline_code_ranges(&segments),
                tall_row_height: inline_code_row_height(&self.theme, font_size),
                wrap_width: self.wrap_width,
                bold: level.is_some(),
                box_model,
                theme: &self.theme,
            },
            window,
        )
    }

    pub(crate) fn line_decor(&self, line: usize) -> LineDecor {
        decor_of(line, &self.decorations)
    }

    pub(crate) fn layout_for_line(&mut self, line: usize, window: &mut Window) -> Rc<LineLayout> {
        if let Some(layout) = self.layouts.get(&line) {
            return layout.clone();
        }
        if self.layouts.len() >= LAYOUT_CACHE_LIMIT {
            self.layouts.clear();
        }
        let layout = Rc::new(self.shape_line(line, window));
        self.layouts.insert(line, layout.clone());
        layout
    }

    pub(crate) fn max_scroll(&self) -> Pixels {
        let viewport = self
            .viewport
            .map_or(0.0, |bounds| f32::from(bounds.size.height));
        px((self.heights.total() - viewport).max(0.0))
    }

    pub(crate) fn scroll_by(&mut self, delta: Pixels, cx: &mut Context<Self>) {
        let target = (self.scroll_top + delta)
            .max(px(0.0))
            .min(self.max_scroll());
        if target != self.scroll_top {
            self.scroll_top = target;
            cx.notify();
        }
    }
}

pub(crate) fn line_height_of(theme: &EditorTheme, font_size: Pixels) -> Pixels {
    font_size * theme.line_height
}

pub(crate) fn inline_code_row_height(theme: &EditorTheme, font_size: Pixels) -> Pixels {
    font_size * theme.inline_code_line_height + theme.inline_code_border * 2.0
}

impl Render for Editor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        crate::view::editor_root(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blink::BLINK_INTERVAL;
    use gpui::{TestAppContext, VisualTestContext};
    use std::time::Duration;

    fn open<'a>(
        cx: &'a mut TestAppContext,
        text: &str,
    ) -> (gpui::Entity<Editor>, &'a mut VisualTestContext) {
        let text = text.to_owned();
        cx.add_window_view(move |_, cx| Editor::new(&text, EditorTheme::light(), cx))
    }

    #[gpui::test]
    fn the_caret_blinks_and_stops_for_stale_timers(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "x");
        assert!(editor.read_with(cx, |editor, _| editor.caret_visible));
        cx.executor().advance_clock(BLINK_INTERVAL);
        assert!(!editor.read_with(cx, |editor, _| editor.caret_visible));
        cx.executor().advance_clock(BLINK_INTERVAL);
        assert!(editor.read_with(cx, |editor, _| editor.caret_visible));
        cx.executor().advance_clock(BLINK_INTERVAL);
        assert!(!editor.read_with(cx, |editor, _| editor.caret_visible));
    }

    #[gpui::test]
    fn restarting_the_blink_shows_the_caret_and_cancels_the_old_timer(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "x");
        cx.executor().advance_clock(BLINK_INTERVAL);
        editor.update(cx, Editor::restart_blink);
        assert!(editor.read_with(cx, |editor, _| editor.caret_visible));
        cx.executor().advance_clock(Duration::from_millis(400));
        assert!(editor.read_with(cx, |editor, _| editor.caret_visible));
        cx.executor().advance_clock(Duration::from_millis(400));
        assert!(!editor.read_with(cx, |editor, _| editor.caret_visible));
    }

    #[gpui::test]
    fn typing_keeps_the_caret_visible(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "");
        cx.executor().advance_clock(BLINK_INTERVAL);
        editor.update(cx, |editor, cx| editor.set_selection(0, 0, cx));
        assert!(editor.read_with(cx, |editor, _| editor.caret_visible));
    }

    #[gpui::test]
    fn layouts_are_cached_until_invalidated(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "a\nb");
        editor.update_in(cx, |editor, window, _| {
            editor.clear_layouts();
            let first = editor.layout_for_line(0, window);
            let again = editor.layout_for_line(0, window);
            assert!(Rc::ptr_eq(&first, &again));
            editor.invalidate_layouts();
            let fresh = editor.layout_for_line(0, window);
            assert!(!Rc::ptr_eq(&first, &fresh));
            let second = editor.layout_for_line(1, window);
            assert!(Rc::ptr_eq(&fresh, &editor.layout_for_line(0, window)));
            assert!(Rc::ptr_eq(&second, &editor.layout_for_line(1, window)));
        });
    }

    #[gpui::test]
    fn the_layout_cache_is_bounded(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(LAYOUT_CACHE_LIMIT + 10));
        let size = editor.update_in(cx, |editor, window, _| {
            editor.clear_layouts();
            for line in 0..LAYOUT_CACHE_LIMIT + 5 {
                editor.layout_for_line(line, window);
            }
            editor.layouts.len()
        });
        assert!(size <= LAYOUT_CACHE_LIMIT);
        assert!(size > 0);
    }

    #[gpui::test]
    fn heading_lines_are_shaped_larger(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "# Title\nplain");
        let (heading, plain) = editor.update_in(cx, |editor, window, _| {
            (
                f32::from(editor.shape_line(0, window).height()),
                f32::from(editor.shape_line(1, window).height()),
            )
        });
        assert!((heading - 33.64).abs() < 0.05);
        assert!((plain - 16.82).abs() < 0.05);
    }

    #[gpui::test]
    fn the_scroll_range_ends_where_the_document_ends(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(99));
        let (max, expected) = editor.update_in(cx, |editor, window, _| {
            let viewport = f32::from(window.viewport_size().height);
            (f32::from(editor.max_scroll()), 100.0 * 16.82 - viewport)
        });
        assert!((max - expected).abs() < 0.05);
    }

    #[gpui::test]
    fn scrolling_clamps_to_both_ends(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(99));
        editor.update(cx, |editor, cx| editor.scroll_by(px(1.0e6), cx));
        let bottom = editor.read_with(cx, |editor, _| editor.scroll_top());
        assert_eq!(
            bottom,
            editor.read_with(cx, |editor, _| editor.max_scroll())
        );
        editor.update(cx, |editor, cx| editor.scroll_by(px(-1.0e6), cx));
        assert_eq!(
            editor.read_with(cx, |editor, _| editor.scroll_top()),
            px(0.0)
        );
    }

    #[gpui::test]
    fn replacing_the_text_refreshes_highlights_and_line_heights(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "");
        editor.update(cx, |editor, cx| editor.set_text("- [ ] a\nb\nc", cx));
        let (marker, lines) = editor.read_with(cx, |editor, _| {
            (
                crate::task_click::task_marker_at(&editor.spans, 3),
                editor.heights.line_count(),
            )
        });
        assert_eq!(marker, Some(2..5));
        assert_eq!(lines, 3);
    }

    #[gpui::test]
    fn changing_the_theme_remeasures_every_line(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(120));
        cx.run_until_parked();
        let mut theme = EditorTheme::light();
        theme.line_height = 3.0;
        editor.update(cx, |editor, cx| editor.set_theme(theme.clone(), cx));
        cx.run_until_parked();
        let (stored, total) = editor.read_with(cx, |editor, _| {
            (editor.theme().clone(), f32::from(editor.max_scroll()))
        });
        assert_eq!(stored, theme);
        let viewport = cx.update(|window, _| f32::from(window.viewport_size().height));
        assert!((total - (121.0 * 14.5 * 3.0 - viewport).max(0.0)).abs() < 0.5);
    }

    #[gpui::test]
    fn toggling_line_numbers_is_remembered(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "x");
        editor.update(cx, |editor, cx| editor.set_line_numbers(true, cx));
        assert!(editor.read_with(cx, |editor, _| editor.line_numbers()));
        editor.update(cx, |editor, cx| editor.set_line_numbers(false, cx));
        assert!(!editor.read_with(cx, |editor, _| editor.line_numbers()));
    }

    #[gpui::test]
    fn undo_and_redo_only_run_when_there_is_history(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "abc");
        editor.update(cx, |editor, cx| {
            editor.undo(cx);
            editor.redo(cx);
        });
        assert_eq!(editor.read_with(cx, |editor, _| editor.text()), "abc");
    }
}
