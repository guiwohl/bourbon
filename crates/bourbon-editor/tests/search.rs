use bourbon_editor::{
    Editor, EditorTheme, SearchOptions, SearchPanel, bind_editor_keys, bind_search_keys,
    bind_text_field_keys, match_count_label,
};
use bourbon_ui::{Appearance, Theme};
use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    TestAppContext, VisualTestContext, Window, div, px,
};

struct Host {
    panel: Entity<SearchPanel>,
    editor: Entity<Editor>,
}

impl Render for Host {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().w(px(600.0)).child(self.panel.clone()))
            .child(div().flex_1().child(self.editor.clone()))
    }
}

struct Opened<'a> {
    panel: Entity<SearchPanel>,
    editor: Entity<Editor>,
    cx: &'a mut VisualTestContext,
}

fn open<'a>(cx: &'a mut TestAppContext, text: &str) -> Opened<'a> {
    cx.update(|cx| {
        Theme::install(Appearance::Light, cx);
        bind_editor_keys(cx);
        bind_text_field_keys(cx);
        bind_search_keys(cx);
    });
    let text = text.to_owned();
    let (host, cx) = cx.add_window_view(move |window, cx| {
        let editor = cx.new(|cx| Editor::new(&text, EditorTheme::light(), cx));
        let panel = cx.new(|cx| SearchPanel::new(editor.clone(), window, cx));
        Host { panel, editor }
    });
    let (panel, editor) = host.read_with(cx, |host, _| (host.panel.clone(), host.editor.clone()));
    cx.run_until_parked();
    Opened { panel, editor, cx }
}

impl Opened<'_> {
    fn show(&mut self, replacing: bool) {
        self.panel.update_in(self.cx, |panel, window, cx| {
            panel.open(replacing, window, cx);
        });
        self.cx.run_until_parked();
    }

    fn type_text(&mut self, text: &str) {
        self.cx.simulate_input(text);
        self.cx.run_until_parked();
    }

    fn keys(&mut self, keystrokes: &str) {
        self.cx.simulate_keystrokes(keystrokes);
        self.cx.run_until_parked();
    }

    fn selection(&mut self) -> std::ops::Range<usize> {
        self.editor
            .read_with(self.cx, |editor, _| editor.selected_range())
    }

    fn text(&mut self) -> String {
        self.editor.read_with(self.cx, |editor, _| editor.text())
    }

    fn matches(&mut self) -> Vec<std::ops::Range<usize>> {
        self.panel
            .read_with(self.cx, |panel, _| panel.matches().to_vec())
    }

    fn label(&mut self) -> String {
        self.panel.read_with(self.cx, SearchPanel::count_label)
    }

    fn set_replacement(&mut self, text: &str) {
        let field = self
            .panel
            .read_with(self.cx, |panel, _| panel.replace_field().clone());
        field.update(self.cx, |field, cx| field.set_text(text, cx));
    }

    fn body_focused(&mut self) -> bool {
        let editor = self.editor.clone();
        self.cx
            .update(|window, cx| editor.read(cx).focus_handle().is_focused(window))
    }

    fn find_field_focused(&mut self) -> bool {
        let panel = self.panel.clone();
        self.cx.update(|window, cx| {
            let field = panel.read(cx).find_field().clone();
            gpui::Focusable::focus_handle(field.read(cx), cx).is_focused(window)
        })
    }
}

#[gpui::test]
fn opening_focuses_the_find_field(cx: &mut TestAppContext) {
    let mut opened = open(cx, "alpha");
    opened.show(false);
    assert!(
        opened
            .panel
            .read_with(opened.cx, |panel, _| panel.is_open())
    );
    assert!(
        !opened
            .panel
            .read_with(opened.cx, |panel, _| panel.is_replacing())
    );
    assert!(opened.find_field_focused());
}

#[gpui::test]
fn typing_finds_every_match_and_selects_the_first_after_the_caret(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab x ab x ab");
    opened
        .editor
        .update(opened.cx, |editor, cx| editor.set_selection(3, 3, cx));
    opened.show(false);
    opened.type_text("ab");
    assert_eq!(opened.matches(), vec![0..2, 5..7, 10..12]);
    assert_eq!(opened.selection(), 5..7);
    assert_eq!(opened.label(), "2 of 3");
}

#[gpui::test]
fn enter_and_shift_enter_walk_the_matches_and_wrap(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab ab ab");
    opened.show(false);
    opened.type_text("ab");
    assert_eq!(opened.selection(), 0..2);
    opened.keys("enter");
    assert_eq!(opened.selection(), 3..5);
    opened.keys("enter enter");
    assert_eq!(opened.selection(), 0..2);
    opened.keys("shift-enter");
    assert_eq!(opened.selection(), 6..8);
    assert_eq!(opened.label(), "3 of 3");
    opened.keys("ctrl-shift-g");
    assert_eq!(opened.selection(), 3..5);
    opened.keys("ctrl-g");
    assert_eq!(opened.selection(), 6..8);
}

#[gpui::test]
fn escape_closes_clears_and_returns_focus_to_the_body(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab ab");
    opened.show(false);
    opened.type_text("ab");
    opened.keys("escape");
    assert!(
        !opened
            .panel
            .read_with(opened.cx, |panel, _| panel.is_open())
    );
    assert_eq!(opened.matches(), Vec::<std::ops::Range<usize>>::new());
    assert!(opened.body_focused());
}

#[gpui::test]
fn the_options_change_what_matches(cx: &mut TestAppContext) {
    let mut opened = open(cx, "Cat cat concat a.t");
    opened.show(false);
    opened.type_text("cat");
    assert_eq!(opened.matches(), vec![0..3, 4..7, 11..14]);
    opened.keys("alt-c");
    assert_eq!(opened.matches(), vec![4..7, 11..14]);
    opened.keys("alt-w");
    assert_eq!(opened.matches(), vec![4..7]);
    assert_eq!(
        opened
            .panel
            .read_with(opened.cx, |panel, _| panel.options()),
        SearchOptions {
            match_case: true,
            regex: false,
            whole_word: true,
        }
    );
    opened.keys("alt-c alt-w alt-r");
    opened.keys("ctrl-a backspace");
    opened.type_text("a.t");
    assert_eq!(opened.matches(), vec![15..18]);
    opened.keys("ctrl-a backspace");
    opened.type_text("c.t");
    assert_eq!(opened.matches(), vec![0..3, 4..7, 11..14]);
}

#[gpui::test]
fn an_invalid_regex_says_so(cx: &mut TestAppContext) {
    let mut opened = open(cx, "a(b");
    opened.show(false);
    opened.keys("alt-r");
    opened.type_text("(");
    assert_eq!(opened.matches(), Vec::<std::ops::Range<usize>>::new());
    assert_eq!(opened.label(), "Invalid pattern");
}

#[gpui::test]
fn the_selected_word_prefills_the_find_field(cx: &mut TestAppContext) {
    let mut opened = open(cx, "one two one");
    opened
        .editor
        .update(opened.cx, |editor, cx| editor.set_selection(0, 3, cx));
    opened.show(false);
    assert_eq!(opened.matches(), vec![0..3, 8..11]);
    assert_eq!(opened.selection(), 0..3);
    opened.type_text("two");
    assert_eq!(opened.matches(), vec![4..7]);
}

#[gpui::test]
fn a_multi_line_selection_does_not_prefill(cx: &mut TestAppContext) {
    let mut opened = open(cx, "a\nb ab");
    opened
        .editor
        .update(opened.cx, |editor, cx| editor.set_selection(0, 3, cx));
    opened.show(false);
    assert_eq!(opened.matches(), Vec::<std::ops::Range<usize>>::new());
    assert_eq!(opened.label(), "No results");
}

#[gpui::test]
fn replace_swaps_the_current_match_and_moves_on(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab ab ab");
    opened.show(true);
    assert!(
        opened
            .panel
            .read_with(opened.cx, |panel, _| panel.is_replacing())
    );
    opened.type_text("ab");
    opened.set_replacement("X");
    opened.panel.update(opened.cx, SearchPanel::replace_current);
    opened.cx.run_until_parked();
    assert_eq!(opened.text(), "X ab ab");
    assert_eq!(opened.selection(), 2..4);
    assert_eq!(opened.matches(), vec![2..4, 5..7]);
}

#[gpui::test]
fn replace_without_a_current_match_only_selects_the_next(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab ab");
    opened.show(true);
    opened.type_text("ab");
    opened
        .editor
        .update(opened.cx, |editor, cx| editor.set_selection(1, 1, cx));
    opened.set_replacement("X");
    opened.panel.update(opened.cx, SearchPanel::replace_current);
    assert_eq!(opened.text(), "ab ab");
    assert_eq!(opened.selection(), 3..5);
}

#[gpui::test]
fn replace_all_and_replace_are_each_one_undo_step(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab ab ab");
    opened.show(true);
    opened.type_text("ab");
    opened.set_replacement("cd");
    opened.keys("ctrl-enter");
    assert_eq!(opened.text(), "cd cd cd");
    assert_eq!(opened.matches(), Vec::<std::ops::Range<usize>>::new());
    opened.panel.update(opened.cx, SearchPanel::replace_current);
    opened.panel.update(opened.cx, SearchPanel::replace_all);
    assert_eq!(opened.text(), "cd cd cd");
    opened.keys("escape");
    opened.keys("ctrl-z");
    assert_eq!(opened.text(), "ab ab ab");
}

#[gpui::test]
fn editing_the_body_refreshes_the_matches(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab");
    opened.show(false);
    opened.type_text("ab");
    opened.editor.update_in(opened.cx, |editor, window, cx| {
        window.focus(editor.focus_handle(), cx);
        editor.set_selection(2, 2, cx);
    });
    opened.type_text(" ab");
    assert_eq!(opened.matches(), vec![0..2, 3..5]);
}

#[gpui::test]
fn refreshing_a_closed_panel_finds_nothing(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab");
    opened.show(false);
    opened.type_text("ab");
    opened.keys("escape");
    opened.panel.update(opened.cx, SearchPanel::refresh);
    assert_eq!(opened.matches(), Vec::<std::ops::Range<usize>>::new());
}

#[gpui::test]
fn ctrl_enter_from_the_replace_field_replaces_all(cx: &mut TestAppContext) {
    let mut opened = open(cx, "ab ab");
    opened.show(true);
    opened.type_text("ab");
    let field = opened
        .panel
        .read_with(opened.cx, |panel, _| panel.replace_field().clone());
    field.update_in(opened.cx, |field, window, cx| field.focus(window, cx));
    opened.type_text("z");
    opened.keys("enter");
    assert_eq!(opened.text(), "z ab");
    opened.keys("ctrl-enter");
    assert_eq!(opened.text(), "z z");
}

#[test]
fn the_count_label_reads_n_of_m() {
    assert_eq!(match_count_label(Some(1), 4, true), "2 of 4");
    assert_eq!(match_count_label(None, 4, true), "0 of 4");
    assert_eq!(match_count_label(None, 0, true), "No results");
    assert_eq!(match_count_label(None, 0, false), "Invalid pattern");
}
