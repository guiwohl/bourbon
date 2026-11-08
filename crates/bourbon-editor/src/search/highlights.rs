use std::ops::Range;

use bourbon_text::current_match;
use gpui::{Bounds, Context, Pixels, Point, Window, fill, px};

use crate::editor::Editor;
use crate::layout::LineLayout;
use crate::theme::EditorTheme;

const MATCH_RADIUS_EM: f32 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchRect {
    pub bounds: Bounds<Pixels>,
    pub current: bool,
}

impl Editor {
    pub(crate) fn set_search_matches(
        &mut self,
        matches: Vec<Range<usize>>,
        cx: &mut Context<Self>,
    ) {
        self.search_matches = matches;
        cx.notify();
    }

    pub(crate) fn match_rects(
        &self,
        line: Range<usize>,
        layout: &LineLayout,
        origin: Point<Pixels>,
    ) -> Vec<MatchRect> {
        let current = current_match(&self.search_matches, self.state.selection.main());
        self.search_matches
            .iter()
            .enumerate()
            .take_while(|(_, found)| found.start <= line.end)
            .flat_map(|(index, found)| {
                let local = found.start.saturating_sub(line.start)
                    ..found.end.min(line.end).saturating_sub(line.start);
                let is_current = current == Some(index);
                layout
                    .selection_rects(&local, found.end > line.end, self.theme.font_size * 0.5)
                    .into_iter()
                    .map(move |rect| MatchRect {
                        bounds: Bounds::new(origin + rect.origin, rect.size),
                        current: is_current,
                    })
            })
            .collect()
    }
}

pub(crate) fn paint_match_rects(rects: &[MatchRect], theme: &EditorTheme, window: &mut Window) {
    let radius = theme.font_size * MATCH_RADIUS_EM;
    for rect in rects {
        let quad = if rect.current {
            fill(rect.bounds, theme.search_match_current)
                .border_widths(px(1.0))
                .border_color(theme.search_match_outline)
        } else {
            fill(rect.bounds, theme.search_match)
        };
        window.paint_quad(quad.corner_radii(radius));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext, point, size};

    const CHAR: f32 = 7.8;
    const LINE: f32 = 19.5;

    fn open<'a>(
        cx: &'a mut TestAppContext,
        text: &str,
    ) -> (gpui::Entity<Editor>, &'a mut VisualTestContext) {
        let text = text.to_owned();
        cx.add_window_view(move |_, cx| Editor::new(&text, EditorTheme::light(), cx))
    }

    fn frame_matches(
        editor: &gpui::Entity<Editor>,
        cx: &mut VisualTestContext,
        matches: Vec<Range<usize>>,
        selection: Range<usize>,
    ) -> Vec<MatchRect> {
        editor.update_in(cx, |editor, window, cx| {
            editor.set_search_matches(matches, cx);
            editor.set_selection(selection.start, selection.end, cx);
            let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(400.0), px(300.0)));
            editor.build_frame(bounds, window).search_matches
        })
    }

    fn near(actual: Pixels, expected: f32) -> bool {
        (f32::from(actual) - expected).abs() < 0.05
    }

    #[gpui::test]
    fn every_visible_match_gets_a_rectangle_and_only_the_selected_one_is_current(
        cx: &mut TestAppContext,
    ) {
        let (editor, cx) = open(cx, "ab ab\nxx ab");
        let rects = frame_matches(&editor, cx, vec![0..2, 3..5, 9..11], 3..5);
        assert_eq!(rects.len(), 3);
        assert_eq!(
            rects.iter().map(|rect| rect.current).collect::<Vec<_>>(),
            vec![false, true, false]
        );
        let left = f32::from(rects[0].bounds.left());
        assert!(near(rects[1].bounds.left(), left + 3.0 * CHAR));
        assert!(near(rects[1].bounds.size.width, 2.0 * CHAR));
        assert!(near(
            rects[2].bounds.top(),
            f32::from(rects[0].bounds.top()) + LINE
        ));
    }

    #[gpui::test]
    fn a_match_across_a_line_break_marks_both_of_its_lines(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab\ncd\nef");
        let rects = frame_matches(&editor, cx, vec![1..4, 7..8], 0..0);
        assert_eq!(rects.len(), 3);
        assert!(rects.iter().all(|rect| !rect.current));
        assert!(near(rects[0].bounds.size.width, CHAR + 6.5));
        assert!(near(rects[1].bounds.size.width, CHAR));
    }

    #[gpui::test]
    fn a_match_of_only_the_line_break_marks_the_end_of_its_line(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab\ncd");
        let rects = frame_matches(&editor, cx, vec![2..3, 4..5], 4..5);
        assert_eq!(rects.len(), 2);
        assert!(near(
            rects[0].bounds.left(),
            f32::from(rects[1].bounds.left()) + CHAR
        ));
        assert!(near(rects[0].bounds.size.width, 6.5));
        assert_eq!(
            rects.iter().map(|rect| rect.current).collect::<Vec<_>>(),
            vec![false, true]
        );
    }

    #[gpui::test]
    fn no_rectangles_without_matches(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx, "ab");
        assert_eq!(
            frame_matches(&editor, cx, Vec::new(), 0..0),
            Vec::<MatchRect>::new()
        );
    }
}
