use std::ops::Range;

use bourbon_text::{
    EditorState, Search, SearchQuery, SelectionRange, Transaction, current_match, next_match,
    previous_match,
};
use gpui::{App, AppContext as _, Context, Entity, IntoElement, Render, Subscription, Window};

use crate::editor::{Editor, EditorEvent};
use crate::text_field::{TextField, TextFieldEvent};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    pub match_case: bool,
    pub regex: bool,
    pub whole_word: bool,
}

pub struct SearchPanel {
    pub(super) editor: Entity<Editor>,
    pub(super) find_field: Entity<TextField>,
    pub(super) replace_field: Entity<TextField>,
    pub(super) options: SearchOptions,
    pub(super) replacing: bool,
    open: bool,
    search: Option<Search>,
    matches: Vec<Range<usize>>,
    _subscriptions: Vec<Subscription>,
}

pub fn match_count_label(current: Option<usize>, total: usize, valid: bool) -> String {
    if !valid {
        return "Invalid pattern".to_owned();
    }
    if total == 0 {
        return "No results".to_owned();
    }
    let position = current.map_or(0, |index| index + 1);
    format!("{position} of {total}")
}

impl SearchPanel {
    pub fn new(editor: Entity<Editor>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let find_field = cx.new(|cx| TextField::new(cx).placeholder("Find"));
        let replace_field = cx.new(|cx| TextField::new(cx).placeholder("Replace"));
        let subscriptions = vec![
            cx.subscribe(&editor, |panel, _, event: &EditorEvent, cx| {
                if matches!(event, EditorEvent::Changed(_)) {
                    panel.refresh(cx);
                }
            }),
            cx.subscribe_in(
                &find_field,
                window,
                |panel, _, event: &TextFieldEvent, window, cx| match event {
                    TextFieldEvent::Changed(_) => panel.query_changed(cx),
                    TextFieldEvent::Submit => panel.find_next(cx),
                    TextFieldEvent::Cancel => panel.close(window, cx),
                },
            ),
            cx.subscribe_in(
                &replace_field,
                window,
                |panel, _, event: &TextFieldEvent, window, cx| match event {
                    TextFieldEvent::Changed(_) => {}
                    TextFieldEvent::Submit => panel.replace_current(cx),
                    TextFieldEvent::Cancel => panel.close(window, cx),
                },
            ),
        ];
        Self {
            editor,
            find_field,
            replace_field,
            options: SearchOptions::default(),
            replacing: false,
            open: false,
            search: None,
            matches: Vec::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_replacing(&self) -> bool {
        self.open && self.replacing
    }

    pub fn options(&self) -> SearchOptions {
        self.options
    }

    pub fn matches(&self) -> &[Range<usize>] {
        &self.matches
    }

    pub fn find_field(&self) -> &Entity<TextField> {
        &self.find_field
    }

    pub fn replace_field(&self) -> &Entity<TextField> {
        &self.replace_field
    }

    pub fn open(&mut self, replacing: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.replacing = replacing;
        if let Some(selected) = self.single_line_selection(cx) {
            self.find_field
                .update(cx, |field, cx| field.set_text(&selected, cx));
        }
        self.find_field.update(cx, |field, cx| {
            field.select_all(cx);
            field.focus(window, cx);
        });
        self.query_changed(cx);
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = false;
        self.matches.clear();
        self.editor.update(cx, |editor, cx| {
            editor.set_search_matches(Vec::new(), cx);
            window.focus(&editor.focus_handle, cx);
        });
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        let text = self.editor_state(cx).text;
        self.matches = self
            .search
            .as_ref()
            .map(|search| search.find_all(&text))
            .unwrap_or_default();
        let matches = self.matches.clone();
        self.editor
            .update(cx, |editor, cx| editor.set_search_matches(matches, cx));
        cx.notify();
    }

    pub fn set_options(&mut self, options: SearchOptions, cx: &mut Context<Self>) {
        self.options = options;
        self.query_changed(cx);
    }

    pub fn toggle_match_case(&mut self, cx: &mut Context<Self>) {
        let options = SearchOptions {
            match_case: !self.options.match_case,
            ..self.options
        };
        self.set_options(options, cx);
    }

    pub fn toggle_regex(&mut self, cx: &mut Context<Self>) {
        let options = SearchOptions {
            regex: !self.options.regex,
            ..self.options
        };
        self.set_options(options, cx);
    }

    pub fn toggle_whole_word(&mut self, cx: &mut Context<Self>) {
        let options = SearchOptions {
            whole_word: !self.options.whole_word,
            ..self.options
        };
        self.set_options(options, cx);
    }

    pub fn find_next(&mut self, cx: &mut Context<Self>) {
        let selection = self.editor_state(cx).selection.main();
        self.select_match(next_match(&self.matches, selection), cx);
    }

    pub fn find_previous(&mut self, cx: &mut Context<Self>) {
        let selection = self.editor_state(cx).selection.main();
        self.select_match(previous_match(&self.matches, selection), cx);
    }

    pub fn replace_current(&mut self, cx: &mut Context<Self>) {
        let state = self.editor_state(cx);
        let Some(index) = current_match(&self.matches, state.selection.main()) else {
            self.find_next(cx);
            return;
        };
        let range = self.matches[index].clone();
        let replacement = self.replace_field.read(cx).text();
        let transaction = self
            .search
            .as_ref()
            .and_then(|search| search.replace_match(&state, range, &replacement));
        self.apply(transaction, cx);
        self.find_next(cx);
    }

    pub fn replace_all(&mut self, cx: &mut Context<Self>) {
        let state = self.editor_state(cx);
        let replacement = self.replace_field.read(cx).text();
        let transaction = self
            .search
            .as_ref()
            .and_then(|search| search.replace_all(&state, &replacement));
        self.apply(transaction, cx);
    }

    pub fn count_label(&self, cx: &App) -> String {
        let selection = self.editor.read(cx).state.selection.main();
        match_count_label(
            current_match(&self.matches, selection),
            self.matches.len(),
            self.search.is_some(),
        )
    }

    fn apply(&mut self, transaction: Option<Transaction>, cx: &mut Context<Self>) {
        if let Some(transaction) = transaction {
            self.editor
                .update(cx, |editor, cx| editor.apply(&transaction, cx));
            self.refresh(cx);
        }
    }

    fn query_changed(&mut self, cx: &mut Context<Self>) {
        let query = SearchQuery {
            pattern: self.find_field.read(cx).text(),
            match_case: self.options.match_case,
            regex: self.options.regex,
            whole_word: self.options.whole_word,
        };
        self.search = Search::new(&query).ok();
        self.refresh(cx);
        let start = self.editor_state(cx).selection.main().from();
        let index = next_match(&self.matches, SelectionRange::cursor(start));
        self.select_match(index, cx);
    }

    fn select_match(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        let Some(found) = index.and_then(|index| self.matches.get(index)).cloned() else {
            return;
        };
        self.editor.update(cx, |editor, cx| {
            editor.set_selection(found.start, found.end, cx);
        });
        cx.notify();
    }

    fn single_line_selection(&self, cx: &App) -> Option<String> {
        let state = &self.editor.read(cx).state;
        let main = state.selection.main();
        let selected = state.text.slice(main.from()..main.to());
        (!selected.is_empty() && !selected.contains('\n')).then_some(selected)
    }

    fn editor_state(&self, cx: &App) -> EditorState {
        self.editor.read(cx).state.clone()
    }
}

impl Render for SearchPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::view::render(self, window, cx)
    }
}
