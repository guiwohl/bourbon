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
        for _ in 0..SCROLL_ATTEMPTS {
            self.measure_unmeasured_lines(window);
            self.scroll_top = self.scroll_top.max(px(0.0)).min(self.max_scroll());
            visible = self.layout_visible_lines(bounds.size.height, window);
            if !self.scroll_to_caret || !self.scroll_caret_into_view(bounds.size.height, window) {
                break;
            }
        }
        self.scroll_to_caret = false;
        self.assemble_frame(bounds, gutter_width, &visible, window)
    }

    fn measure_unmeasured_lines(&mut self, window: &mut Window) {
        let mut budget = MEASURE_BUDGET;
        for line in 0..self.heights.line_count() {
            if budget == 0 {
                window.request_animation_frame();
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
        }
    }
}
