use std::ops::{Range, RangeInclusive};

use bourbon_text::SelectionRange;
use gpui::{Bounds, Context, Pixels, Point, ShapedLine, point, px, size};

use crate::editor::Editor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chevron {
    pub center: Point<Pixels>,
    pub width: Pixels,
    pub folded: bool,
}

pub struct FoldPlaceholder {
    pub bounds: Bounds<Pixels>,
    pub label: ShapedLine,
}

pub const FOLD_PLACEHOLDER_LABEL: &str = "\u{2026}";

pub fn chevron_center_x(gutter_width: Pixels, chevron_width: Pixels) -> Pixels {
    gutter_width - px(1.0) - chevron_width * 1.25
}

pub fn in_chevron_column(x: Pixels, gutter_width: Pixels, chevron_width: Pixels) -> bool {
    x >= chevron_center_x(gutter_width, chevron_width) - chevron_width && x < gutter_width
}

pub fn placeholder_bounds(
    text_end: Point<Pixels>,
    row_middle: Pixels,
    label_width: Pixels,
    font_size: Pixels,
) -> Bounds<Pixels> {
    let padding = font_size * 0.25;
    let height = font_size;
    Bounds::new(
        point(text_end.x + font_size * 0.3, row_middle - height / 2.0),
        size(label_width + padding * 2.0, height),
    )
}

pub fn chevron_points(chevron: &Chevron) -> [Point<Pixels>; 3] {
    let half = chevron.width / 2.0;
    let quarter = chevron.width / 4.0;
    let center = chevron.center;
    if chevron.folded {
        [
            point(center.x - quarter, center.y - half),
            point(center.x + quarter, center.y),
            point(center.x - quarter, center.y + half),
        ]
    } else {
        [
            point(center.x - half, center.y - quarter),
            point(center.x, center.y + quarter),
            point(center.x + half, center.y - quarter),
        ]
    }
}

pub fn innermost_range_at(
    ranges: &[RangeInclusive<usize>],
    line: usize,
) -> Option<&RangeInclusive<usize>> {
    ranges
        .iter()
        .find(|range| *range.start() == line)
        .or_else(|| {
            ranges
                .iter()
                .filter(|range| range.contains(&line))
                .max_by_key(|range| *range.start())
        })
}

impl Editor {
    fn hidden_bytes(&self, lines: &RangeInclusive<usize>) -> Range<usize> {
        self.state.text.line_end(*lines.start())..self.state.text.line_end(*lines.end())
    }

    pub(crate) fn is_line_hidden(&self, line: usize) -> bool {
        self.folds.hides(self.state.text.line_start(line))
    }

    pub(crate) fn is_folded(&self, line: usize) -> bool {
        self.folds.is_folded_at(self.state.text.line_end(line))
    }

    pub(crate) fn is_fold_header(&self, line: usize) -> bool {
        self.fold_ranges.iter().any(|range| *range.start() == line)
    }

    pub(crate) fn chevron_line_at(&self, position: Point<Pixels>) -> Option<usize> {
        let bounds = self.viewport?;
        let width = self.theme.font_size * self.theme.fold_chevron_em;
        let x = position.x - bounds.left();
        if !self.line_numbers || !in_chevron_column(x, self.gutter_width, width) {
            return None;
        }
        let line = self
            .heights
            .line_at(f32::from(position.y - bounds.top() + self.scroll_top));
        self.is_fold_header(line).then_some(line)
    }

    pub fn toggle_fold(&mut self, line: usize, cx: &mut Context<Self>) {
        if self.is_folded(line) {
            self.unfold_line(line, cx);
        } else if let Some(range) = self
            .fold_ranges
            .iter()
            .find(|range| *range.start() == line)
            .cloned()
        {
            self.fold_lines(&range, cx);
        }
    }

    pub(crate) fn fold_at_caret(&mut self, cx: &mut Context<Self>) {
        let line = self.state.text.line_of(self.state.selection.main().head);
        if let Some(range) = innermost_range_at(&self.fold_ranges, line).cloned() {
            self.fold_lines(&range, cx);
        }
    }

    pub(crate) fn unfold_at_caret(&mut self, cx: &mut Context<Self>) {
        let line = self.state.text.line_of(self.state.selection.main().head);
        self.unfold_line(line, cx);
    }

    fn fold_lines(&mut self, range: &RangeInclusive<usize>, cx: &mut Context<Self>) {
        self.folds.fold(self.hidden_bytes(range));
        self.heights.invalidate(*range.start()..*range.end() + 1);
        let main = self.state.selection.main();
        if self.folds.hides(main.head) || self.folds.hides(main.anchor) {
            let header_end = self.state.text.line_end(*range.start());
            self.select(SelectionRange::cursor(header_end), cx);
        }
        cx.notify();
    }

    fn unfold_line(&mut self, line: usize, cx: &mut Context<Self>) {
        let start = self.state.text.line_end(line);
        let ends: Vec<usize> = self
            .folds
            .ranges()
            .iter()
            .filter(|range| range.start == start)
            .map(|range| self.state.text.line_of(range.end))
            .collect();
        if self.folds.unfold(start) {
            let last = ends.into_iter().max().unwrap_or(line);
            self.heights.invalidate(line..last + 1);
            cx.notify();
        }
    }

    pub(crate) fn reveal(&mut self, offset: usize) {
        for range in self.folds.unfold_hiding(offset) {
            let text = &self.state.text;
            self.heights
                .invalidate(text.line_of(range.start)..text.line_of(range.end) + 1);
        }
    }

    pub(crate) fn visible_selection(
        &self,
        range: SelectionRange,
        previous_head: usize,
    ) -> SelectionRange {
        let length = self.state.text.len();
        let head = self
            .folds
            .visible_position(range.head, range.head > previous_head, length);
        let anchor = if range.is_empty() { head } else { range.anchor };
        SelectionRange::new(anchor, head)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::EditorTheme;
    use gpui::{TestAppContext, VisualTestContext, px};

    const SOURCE: &str = "## A\none\ntwo\n\n## B\nthree\n```\ncode\n```\n";

    fn open<'a>(
        cx: &'a mut TestAppContext,
        text: &str,
    ) -> (gpui::Entity<Editor>, &'a mut VisualTestContext) {
        let text = text.to_owned();
        cx.add_window_view(move |_, cx| Editor::new(&text, EditorTheme::light(), cx))
    }

    fn hidden_lines(editor: &Editor) -> Vec<usize> {
        (0..editor.state.text.line_count())
            .filter(|line| editor.is_line_hidden(*line))
            .collect()
    }

    #[test]
    fn an_open_chevron_points_down_and_a_folded_one_right() {
        let open = Chevron {
            center: point(px(10.0), px(20.0)),
            width: px(8.0),
            folded: false,
        };
        assert_eq!(
            chevron_points(&open),
            [
                point(px(6.0), px(18.0)),
                point(px(10.0), px(22.0)),
                point(px(14.0), px(18.0)),
            ]
        );
        let folded = Chevron {
            folded: true,
            ..open
        };
        assert_eq!(
            chevron_points(&folded),
            [
                point(px(8.0), px(16.0)),
                point(px(12.0), px(20.0)),
                point(px(8.0), px(24.0)),
            ]
        );
    }

    #[test]
    fn the_chevron_sits_right_of_the_numbers_and_its_column_is_clickable() {
        let center = chevron_center_x(px(47.0), px(9.0));
        assert!((f32::from(center) - 34.75).abs() < 1e-4);
        assert!(in_chevron_column(px(26.0), px(47.0), px(9.0)));
        assert!(in_chevron_column(px(46.9), px(47.0), px(9.0)));
        assert!(!in_chevron_column(px(25.0), px(47.0), px(9.0)));
        assert!(!in_chevron_column(px(47.0), px(47.0), px(9.0)));
    }

    #[test]
    fn the_placeholder_follows_the_text_centred_on_its_row() {
        let bounds = placeholder_bounds(point(px(100.0), px(4.0)), px(30.0), px(8.0), px(20.0));
        assert_eq!(bounds.origin, point(px(106.0), px(20.0)));
        assert_eq!(bounds.size, size(px(18.0), px(20.0)));
    }

    #[test]
    fn the_range_starting_on_the_line_wins_over_an_enclosing_one() {
        let ranges = [0..=9, 2..=5, 3..=4];
        assert_eq!(innermost_range_at(&ranges, 2), Some(&(2..=5)));
        assert_eq!(innermost_range_at(&ranges, 4), Some(&(3..=4)));
        assert_eq!(innermost_range_at(&ranges, 7), Some(&(0..=9)));
        assert_eq!(innermost_range_at(&ranges, 12), None);
    }

    #[gpui::test]
    fn toggling_a_heading_hides_its_section_and_shows_it_again(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            assert!(editor.is_fold_header(0));
            assert!(!editor.is_fold_header(1));
            editor.toggle_fold(0, cx);
            assert!(editor.is_folded(0));
            assert_eq!(hidden_lines(editor), vec![1, 2]);
            editor.toggle_fold(0, cx);
            assert!(!editor.is_folded(0));
            assert_eq!(hidden_lines(editor), Vec::<usize>::new());
        });
    }

    #[gpui::test]
    fn toggling_a_line_without_a_fold_range_does_nothing(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(1, cx);
            assert_eq!(hidden_lines(editor), Vec::<usize>::new());
        });
    }

    #[gpui::test]
    fn folding_at_the_caret_picks_the_innermost_range(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            let code = editor.state.text.line_start(7);
            editor.set_selection(code, code, cx);
            editor.fold_at_caret(cx);
            assert_eq!(hidden_lines(editor), vec![7, 8]);
            assert_eq!(editor.selected_range(), {
                let header_end = editor.state.text.line_end(6);
                header_end..header_end
            });
            editor.unfold_at_caret(cx);
            assert_eq!(hidden_lines(editor), Vec::<usize>::new());
        });
    }

    #[gpui::test]
    fn unfolding_a_line_that_is_not_folded_keeps_other_folds(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(4, cx);
            editor.set_selection(0, 0, cx);
            editor.unfold_at_caret(cx);
            assert_eq!(hidden_lines(editor), vec![5, 6, 7, 8]);
        });
    }

    #[gpui::test]
    fn hidden_lines_take_no_height(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        let bounds = gpui::Bounds::new(point(px(0.0), px(0.0)), gpui::size(px(400.0), px(400.0)));
        let (open_lines, folded_lines, folded_top) = editor.update_in(cx, |editor, window, cx| {
            let open_lines = editor.build_frame(bounds, window).lines.len();
            editor.toggle_fold(0, cx);
            let frame = editor.build_frame(bounds, window);
            let folded_top = frame.lines[1].origin.y;
            (open_lines, frame.lines.len(), folded_top)
        });
        assert_eq!(folded_lines, open_lines - 2);
        assert!((f32::from(folded_top) - 16.82 * 1.5).abs() < 0.1);
    }

    #[gpui::test]
    fn the_caret_skips_folded_text(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(0, cx);
            let inside = editor.state.text.line_start(1) + 1;
            editor.select(SelectionRange::cursor(inside), cx);
            let after = editor.state.text.line_start(3);
            assert_eq!(editor.selected_range(), after..after);
            editor.move_horizontal(crate::motion::Motion::Left, false, cx);
            let header_end = editor.state.text.line_end(0);
            assert_eq!(editor.selected_range(), header_end..header_end);
        });
    }

    #[gpui::test]
    fn a_selection_keeps_its_anchor_when_its_head_leaves_a_fold(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(0, cx);
            let inside = editor.state.text.line_start(2);
            editor.select(SelectionRange::new(0, inside), cx);
            assert_eq!(editor.selected_range(), 0..editor.state.text.line_start(3));
        });
    }

    #[gpui::test]
    fn selecting_hidden_text_from_outside_unfolds_to_reveal_it(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(0, cx);
            editor.toggle_fold(4, cx);
            let inside = editor.state.text.line_start(1) + 1;
            editor.set_selection(inside, inside + 2, cx);
            assert_eq!(editor.selected_range(), inside..inside + 2);
            assert_eq!(hidden_lines(editor), vec![5, 6, 7, 8]);
        });
    }

    #[gpui::test]
    fn editing_the_folded_text_drops_the_fold(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(0, cx);
            let end = editor.state.text.line_end(0);
            editor.replace_range(end..end + 1, "", cx);
            assert_eq!(hidden_lines(editor), Vec::<usize>::new());
        });
    }

    #[gpui::test]
    fn editing_before_a_fold_keeps_it(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(4, cx);
            editor.replace_range(0..0, "x", cx);
            assert_eq!(hidden_lines(editor), vec![5, 6, 7, 8]);
        });
    }

    #[gpui::test]
    fn replacing_the_text_forgets_the_folds(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(0, cx);
            editor.set_text(SOURCE, cx);
            assert_eq!(hidden_lines(editor), Vec::<usize>::new());
        });
    }

    const LONG_SECTION_LINES: usize = 50;

    fn long_section() -> String {
        format!("## A\n{}## B\nend", "x\n".repeat(LONG_SECTION_LINES))
    }

    fn total_after_frame(editor: &mut Editor, window: &mut gpui::Window) -> f32 {
        let tiny = gpui::Bounds::new(point(px(0.0), px(0.0)), gpui::size(px(400.0), px(20.0)));
        editor.build_frame(tiny, window);
        editor.heights.total()
    }

    fn near(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() < 0.1
    }

    const HEADING: f32 = 21.75 * 1.16;
    const LINE: f32 = 14.5 * 1.16;

    #[gpui::test]
    fn folding_and_unfolding_remeasure_every_hidden_line(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &long_section());
        let full = 2.0 * HEADING + LINE * crate::numeric::count_f32(LONG_SECTION_LINES + 1);
        editor.update_in(cx, |editor, window, cx| {
            assert!(near(total_after_frame(editor, window), full));
            editor.toggle_fold(0, cx);
            assert!(near(
                total_after_frame(editor, window),
                2.0 * HEADING + LINE
            ));
            editor.toggle_fold(0, cx);
            assert!(near(total_after_frame(editor, window), full));
            editor.toggle_fold(0, cx);
            total_after_frame(editor, window);
            let inside = editor.state.text.line_start(LONG_SECTION_LINES);
            editor.set_selection(inside, inside, cx);
            assert!(near(total_after_frame(editor, window), full));
        });
    }

    #[gpui::test]
    fn folding_over_a_selection_anchor_collapses_it_to_the_header(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            let inside = editor.state.text.line_start(2);
            editor.select(SelectionRange::new(inside, 0), cx);
            editor.toggle_fold(0, cx);
            let header_end = editor.state.text.line_end(0);
            assert_eq!(editor.selected_range(), header_end..header_end);
        });
    }

    #[gpui::test]
    fn an_edit_leaving_the_caret_inside_a_fold_puts_it_on_the_header(cx: &mut TestAppContext) {
        use bourbon_text::{Change, ChangeSet, Selection, Transaction, TransactionKind};
        let (editor, cx) = open(cx, SOURCE);
        editor.update(cx, |editor, cx| {
            editor.toggle_fold(0, cx);
            let inside = editor.state.text.line_start(1) + 1;
            let last = editor.state.text.len();
            editor.apply(
                &Transaction::new(
                    ChangeSet::new(vec![Change::insert(last, "z")]),
                    Selection::cursor(inside),
                    TransactionKind::Input,
                ),
                cx,
            );
            let header_end = editor.state.text.line_end(0);
            assert_eq!(editor.selected_range(), header_end..header_end);
            assert_eq!(hidden_lines(editor), vec![1, 2]);
        });
    }

    #[gpui::test]
    fn only_the_chevron_column_of_a_header_toggles(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, SOURCE);
        editor.update_in(cx, |editor, window, cx| {
            editor.set_line_numbers(true, cx);
            let viewport =
                gpui::Bounds::new(point(px(100.0), px(50.0)), gpui::size(px(400.0), px(60.0)));
            editor.build_frame(viewport, window);
            editor.scroll_top = px(10.0);
            let header_top = editor.heights.top(4);
            let y = px(50.0 + header_top - 10.0 + 5.0);
            let at = |x: f32| point(px(100.0 + x), y);
            assert_eq!(editor.chevron_line_at(at(35.0)), Some(4));
            assert_eq!(editor.chevron_line_at(at(20.0)), None);
            assert_eq!(editor.chevron_line_at(at(100.0)), None);
            assert_eq!(editor.chevron_line_at(point(px(135.0), px(70.0))), None);
            editor.set_line_numbers(false, cx);
            assert_eq!(editor.chevron_line_at(at(35.0)), None);
        });
    }
}
