use std::rc::Rc;

use gpui::{Bounds, Pixels, Point, ShapedLine, TextRun, Window, point, px, size};

use crate::editor::{Editor, line_height_of};
use crate::layout::{LineLayout, editor_font};

const MEASURE_BUDGET: usize = 400;
const SCROLL_ATTEMPTS: usize = 3;

pub struct FrameLine {
    pub origin: Point<Pixels>,
    pub height: Pixels,
    pub layout: Rc<LineLayout>,
    pub active: bool,
    pub selections: Vec<Bounds<Pixels>>,
    pub number: Option<(ShapedLine, Point<Pixels>)>,
}

pub struct Frame {
    pub gutter_width: Pixels,
    pub lines: Vec<FrameLine>,
    pub caret: Option<Bounds<Pixels>>,
    pub measuring_pending: bool,
}

impl Editor {
    pub(crate) fn gutter_width(&self, window: &mut Window) -> Pixels {
        if !self.line_numbers {
            return px(0.0);
        }
        let number_size = self.theme.font_size * self.theme.gutter_font_scale;
        let digits = self.state.text.line_count().to_string().len();
        let sample = "0".repeat(digits);
        let width = window
            .text_system()
            .shape_line(sample.into(), number_size, &[self.number_run(digits)], None)
            .width;
        let wanted = width + number_size * self.theme.gutter_padding_em;
        wanted.max(number_size * self.theme.gutter_min_width_em) + px(1.0)
    }

    fn number_run(&self, len: usize) -> TextRun {
        TextRun {
            len,
            font: editor_font(&self.theme, false, false),
            color: self.theme.gutter_text,
            background_color: None,
            underline: None,
            strikethrough: None,
        }
    }

    pub(crate) fn build_frame(&mut self, bounds: Bounds<Pixels>, window: &mut Window) -> Frame {
        let gutter_width = self.gutter_width(window);
        let text_left = gutter_width + self.theme.line_padding;
        let wrap_width = (bounds.size.width - text_left - self.theme.line_padding).max(px(40.0));
        if self.wrap_width != Some(wrap_width) {
            let estimate = line_height_of(&self.theme, self.theme.font_size);
            self.heights.reset(f32::from(estimate));
            self.heights.refresh();
            self.clear_layouts();
            self.wrap_width = Some(wrap_width);
        }
        self.viewport = Some(bounds);
        self.text_left = text_left;

        let mut visible = Vec::new();
        let mut measuring_pending = false;
        for _ in 0..SCROLL_ATTEMPTS {
            measuring_pending = self.measure_unmeasured_lines(window);
            self.scroll_top = self.scroll_top.max(px(0.0)).min(self.max_scroll());
            visible = self.layout_visible_lines(bounds.size.height, window);
            if !self.scroll_to_caret || !self.scroll_caret_into_view(bounds.size.height, window) {
                break;
            }
        }
        self.scroll_to_caret = false;
        self.assemble_frame(bounds, gutter_width, &visible, measuring_pending, window)
    }

    fn measure_unmeasured_lines(&mut self, window: &mut Window) -> bool {
        let mut budget = MEASURE_BUDGET;
        let mut pending = false;
        for line in 0..self.heights.line_count() {
            if budget == 0 {
                pending = true;
                break;
            }
            if self.heights.is_measured(line) {
                continue;
            }
            let height = self.shape_line(line, window).height();
            self.heights.set_measured(line, f32::from(height));
            budget -= 1;
        }
        self.heights.refresh();
        pending
    }

    fn layout_visible_lines(
        &mut self,
        viewport_height: Pixels,
        window: &mut Window,
    ) -> Vec<(usize, Rc<LineLayout>)> {
        let line_count = self.heights.line_count();
        let first = self.heights.line_at(f32::from(self.scroll_top));
        let bottom = f32::from(self.scroll_top + viewport_height);
        let mut visible = Vec::new();
        let mut y = self.heights.top(first);
        for line in first..line_count {
            if y >= bottom {
                break;
            }
            let layout = self.layout_for_line(line, window);
            self.heights.set_measured(line, f32::from(layout.height()));
            y += f32::from(layout.height());
            visible.push((line, layout));
        }
        self.heights.refresh();
        visible
    }

    fn scroll_caret_into_view(&mut self, viewport_height: Pixels, window: &mut Window) -> bool {
        let head = self.state.selection.main().head;
        let line = self.state.text.line_of(head);
        let layout = self.layout_for_line(line, window);
        let position = layout.position_for_index(head - self.state.text.line_start(line));
        let top = px(self.heights.top(line)) + position.y;
        let bottom = top + layout.line_height();
        let previous = self.scroll_top;
        if top < self.scroll_top {
            self.scroll_top = top;
        } else if bottom > self.scroll_top + viewport_height {
            self.scroll_top = bottom - viewport_height;
        }
        self.scroll_top = self.scroll_top.max(px(0.0)).min(self.max_scroll());
        self.scroll_top != previous
    }

    fn assemble_frame(
        &mut self,
        bounds: Bounds<Pixels>,
        gutter_width: Pixels,
        visible: &[(usize, Rc<LineLayout>)],
        measuring_pending: bool,
        window: &mut Window,
    ) -> Frame {
        let main = self.state.selection.main();
        let head_line = self.state.text.line_of(main.head);
        let number_size = self.theme.font_size * self.theme.gutter_font_scale;
        let mut lines = Vec::new();
        let mut caret = None;
        for (line, layout) in visible {
            let line_top = bounds.top() + px(self.heights.top(*line)) - self.scroll_top;
            let origin = point(bounds.left() + self.text_left, line_top);
            let start = self.state.text.line_start(*line);
            let end = self.state.text.line_end(*line);
            let selections = if main.is_empty() || main.to() <= start || main.from() > end {
                Vec::new()
            } else {
                layout
                    .selection_rects(
                        &(main.from().max(start) - start..main.to().min(end) - start),
                        main.to() > end,
                        self.theme.font_size * 0.5,
                    )
                    .into_iter()
                    .map(|rect| Bounds::new(origin + rect.origin, rect.size))
                    .collect()
            };
            if *line == head_line {
                let position = layout.position_for_index(main.head - start);
                caret = Some(Bounds::new(
                    origin + position,
                    size(self.theme.caret_width, layout.line_height()),
                ));
            }
            let number = self.line_numbers.then(|| {
                let text = (line + 1).to_string();
                let shaped = window.text_system().shape_line(
                    text.clone().into(),
                    number_size,
                    &[self.number_run(text.len())],
                    None,
                );
                let x = bounds.left() + gutter_width
                    - number_size * self.theme.gutter_padding_em
                    - shaped.width
                    - px(1.0);
                (shaped, point(x, line_top))
            });
            lines.push(FrameLine {
                origin,
                height: layout.height(),
                layout: layout.clone(),
                active: *line == head_line,
                selections,
                number,
            });
        }
        Frame {
            gutter_width,
            lines,
            caret,
            measuring_pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::EditorTheme;
    use gpui::{TestAppContext, VisualTestContext};

    const CHAR: f32 = 7.8;
    const LINE: f32 = 19.5;

    fn bounds(width: f32, height: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(100.0), px(50.0)), size(px(width), px(height)))
    }

    fn open<'a>(
        cx: &'a mut TestAppContext,
        text: &str,
    ) -> (gpui::Entity<Editor>, &'a mut VisualTestContext) {
        let text = text.to_owned();
        cx.add_window_view(move |_, cx| Editor::new(&text, EditorTheme::light(), cx))
    }

    fn near(actual: Pixels, expected: f32) -> bool {
        (f32::from(actual) - expected).abs() < 0.05
    }

    #[gpui::test]
    fn lines_are_placed_below_each_other_inside_the_bounds(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "a\nb\nc");
        let origins = editor.update_in(cx, |editor, window, _| {
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            frame
                .lines
                .iter()
                .map(|line| (line.origin, line.height, line.active))
                .collect::<Vec<_>>()
        });
        assert_eq!(origins.len(), 3);
        for (index, (origin, height, active)) in origins.iter().enumerate() {
            assert!(near(origin.x, 116.0));
            assert!(near(
                origin.y,
                50.0 + LINE * crate::numeric::count_f32(index)
            ));
            assert!(near(*height, LINE));
            assert_eq!(*active, index == 0);
        }
    }

    #[gpui::test]
    fn the_caret_sits_at_the_head_of_the_selection(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab\ncd");
        let caret = editor.update_in(cx, |editor, window, cx| {
            editor.set_selection(4, 4, cx);
            editor.build_frame(bounds(400.0, 200.0), window).caret
        });
        let caret = caret.expect("caret");
        assert!(near(caret.origin.x, 116.0 + CHAR));
        assert!(near(caret.origin.y, 50.0 + LINE));
        assert!(near(caret.size.width, 2.0));
        assert!(near(caret.size.height, LINE));
    }

    #[gpui::test]
    fn selections_become_one_rectangle_per_line(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab\ncd\nef");
        let rects = editor.update_in(cx, |editor, window, cx| {
            editor.set_selection(1, 7, cx);
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            frame
                .lines
                .iter()
                .map(|line| line.selections.clone())
                .collect::<Vec<_>>()
        });
        assert_eq!(rects[0].len(), 1);
        assert!(near(rects[0][0].origin.x, 116.0 + CHAR));
        assert!(near(rects[0][0].size.width, CHAR + 6.5));
        assert_eq!(rects[1].len(), 1);
        assert!(near(rects[1][0].origin.x, 116.0));
        assert!(near(rects[1][0].origin.y, 50.0 + LINE));
        assert!(near(rects[1][0].size.width, 2.0 * CHAR + 6.5));
        assert_eq!(rects[2].len(), 1);
        assert!(near(rects[2][0].size.width, CHAR));
    }

    #[gpui::test]
    fn no_selection_rectangles_without_a_selection_or_outside_it(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab\ncd\nef");
        let counts = editor.update_in(cx, |editor, window, cx| {
            editor.set_selection(3, 5, cx);
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            frame
                .lines
                .iter()
                .map(|line| line.selections.len())
                .collect::<Vec<_>>()
        });
        assert_eq!(counts, vec![0, 1, 0]);
        let counts = editor.update_in(cx, |editor, window, cx| {
            editor.set_selection(3, 3, cx);
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            frame
                .lines
                .iter()
                .map(|line| line.selections.len())
                .collect::<Vec<_>>()
        });
        assert_eq!(counts, vec![0, 0, 0]);
    }

    #[gpui::test]
    fn a_selection_ending_at_a_line_start_does_not_touch_that_line(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab\ncd");
        let counts = editor.update_in(cx, |editor, window, cx| {
            editor.set_selection(0, 3, cx);
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            frame
                .lines
                .iter()
                .map(|line| line.selections.len())
                .collect::<Vec<_>>()
        });
        assert_eq!(counts, vec![1, 0]);
    }

    #[gpui::test]
    fn the_gutter_is_off_by_default_and_has_no_numbers(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "a\nb");
        let (gutter, numbers) = editor.update_in(cx, |editor, window, _| {
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            (
                frame.gutter_width,
                frame
                    .lines
                    .iter()
                    .filter(|line| line.number.is_some())
                    .count(),
            )
        });
        assert_eq!(gutter, px(0.0));
        assert_eq!(numbers, 0);
    }

    #[gpui::test]
    fn line_numbers_get_a_minimum_width_and_are_right_aligned(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "a\nb");
        let (gutter, line_x, number) = editor.update_in(cx, |editor, window, cx| {
            editor.set_line_numbers(true, cx);
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            let (shaped, origin) = frame.lines[1].number.as_ref().expect("number");
            (
                frame.gutter_width,
                frame.lines[0].origin.x,
                (shaped.width, origin.x, origin.y),
            )
        });
        assert!(near(gutter, 9.1 * 3.0 + 1.0));
        assert!(near(line_x, 100.0 + 9.1 * 3.0 + 1.0 + 16.0));
        assert!(near(number.0, 9.1 * 0.6));
        assert!(near(
            number.1,
            100.0 + f32::from(gutter) - 9.1 - 9.1 * 0.6 - 1.0
        ));
        assert!(near(number.2, 50.0 + LINE));
    }

    #[gpui::test]
    fn the_gutter_widens_for_many_digits(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(1500));
        let gutter = editor.update_in(cx, |editor, window, cx| {
            editor.set_line_numbers(true, cx);
            editor
                .build_frame(bounds(400.0, 200.0), window)
                .gutter_width
        });
        assert!(near(gutter, 4.0 * 9.1 * 0.6 + 9.1 + 1.0));
    }

    #[gpui::test]
    fn only_the_lines_in_the_viewport_are_laid_out(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(200));
        let (first, count, y) = editor.update_in(cx, |editor, window, _| {
            editor.scroll_top = px(100.0);
            let frame = editor.build_frame(bounds(400.0, 200.0), window);
            let first = frame.lines.first().expect("first line");
            (
                first.origin.y,
                frame.lines.len(),
                frame.lines.last().expect("last").origin.y,
            )
        });
        assert!(near(first, 50.0 + 5.0 * LINE - 100.0));
        assert_eq!(count, 11);
        assert!(near(y, 50.0 + 15.0 * LINE - 100.0));
    }

    #[gpui::test]
    fn scrolling_past_the_end_is_clamped_to_the_document(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(99));
        let scroll = editor.update_in(cx, |editor, window, _| {
            editor.scroll_top = px(100_000.0);
            editor.build_frame(bounds(400.0, 200.0), window);
            editor.scroll_top
        });
        assert!(near(scroll, 100.0 * LINE - 200.0));
    }

    #[gpui::test]
    fn a_caret_below_the_viewport_scrolls_to_its_bottom_edge(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(200));
        let scroll = editor.update_in(cx, |editor, window, cx| {
            editor.set_selection(200, 200, cx);
            editor.build_frame(bounds(400.0, 200.0), window);
            editor.scroll_top
        });
        assert!(near(scroll, 100.0 * LINE + LINE - 200.0));
    }

    #[gpui::test]
    fn a_caret_above_the_viewport_scrolls_to_its_top_edge(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(200));
        let scroll = editor.update_in(cx, |editor, window, cx| {
            editor.scroll_top = px(1000.0);
            editor.set_selection(20, 20, cx);
            editor.build_frame(bounds(400.0, 200.0), window);
            editor.scroll_top
        });
        assert!(near(scroll, 10.0 * LINE));
    }

    #[gpui::test]
    fn a_visible_caret_leaves_the_scroll_alone(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(200));
        let scroll = editor.update_in(cx, |editor, window, cx| {
            editor.scroll_top = px(100.0);
            editor.set_selection(12, 12, cx);
            editor.build_frame(bounds(400.0, 200.0), window);
            editor.scroll_top
        });
        assert!(near(scroll, 100.0));
    }

    #[gpui::test]
    fn the_caret_only_scrolls_when_asked_to(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(200));
        let scroll = editor.update_in(cx, |editor, window, _| {
            editor.scroll_top = px(1000.0);
            editor.build_frame(bounds(400.0, 200.0), window);
            editor.scroll_top
        });
        assert!(near(scroll, 1000.0));
    }

    #[gpui::test]
    fn measuring_is_spread_over_frames(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(1000));
        let (early, late, unmeasured_after_two) = editor.update_in(cx, |editor, window, _| {
            editor.invalidate_all_heights();
            editor.build_frame(bounds(400.0, 200.0), window);
            let early = editor.heights.is_measured(399);
            let late = editor.heights.is_measured(500);
            editor.build_frame(bounds(400.0, 200.0), window);
            (early, late, editor.heights.is_measured(500))
        });
        assert!(early);
        assert!(!late);
        assert!(unmeasured_after_two);
    }

    #[gpui::test]
    fn the_frame_reports_pending_measuring_only_when_lines_remain(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(1000));
        let (busy, idle) = editor.update_in(cx, |editor, window, _| {
            editor.invalidate_all_heights();
            let busy = editor
                .build_frame(bounds(400.0, 200.0), window)
                .measuring_pending;
            editor.build_frame(bounds(400.0, 200.0), window);
            editor.build_frame(bounds(400.0, 200.0), window);
            (
                busy,
                editor
                    .build_frame(bounds(400.0, 200.0), window)
                    .measuring_pending,
            )
        });
        assert!(busy);
        assert!(!idle);
    }

    #[gpui::test]
    fn a_new_width_forgets_the_measured_heights(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, &"x\n".repeat(50));
        let measured = editor.update_in(cx, |editor, window, _| {
            editor.build_frame(bounds(400.0, 200.0), window);
            let before = editor.heights.is_measured(49);
            editor.build_frame(bounds(300.0, 200.0), window);
            (before, editor.wrap_width)
        });
        assert!(measured.0);
        assert!(near(measured.1.expect("wrap width"), 300.0 - 32.0));
    }

    #[gpui::test]
    fn the_wrap_width_never_gets_narrower_than_forty_pixels(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "x");
        let width = editor.update_in(cx, |editor, window, _| {
            editor.build_frame(bounds(10.0, 200.0), window);
            editor.wrap_width
        });
        assert!(near(width.expect("wrap width"), 40.0));
    }
}
