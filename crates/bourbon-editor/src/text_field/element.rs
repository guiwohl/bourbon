use std::ops::Range;

use bourbon_ui::{Colors, ThemeExt as _, Tokens};
use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, Font,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Point, SharedString, Style,
    TextAlign, TextRun, UnderlineStyle, Window, fill, point, px, relative, size,
};

use super::TextField;
use super::geometry::{LineGeometry, revealing_scroll};

const CARET_WIDTH: Pixels = px(1.0);

pub(crate) struct TextFieldElement {
    field: Entity<TextField>,
}

impl TextFieldElement {
    pub(crate) fn new(field: Entity<TextField>) -> Self {
        Self { field }
    }
}

impl IntoElement for TextFieldElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub(crate) struct TextFieldFrame {
    line_origin: Point<Pixels>,
    selection: Option<Bounds<Pixels>>,
    caret: Option<Bounds<Pixels>>,
}

impl TextFieldFrame {
    fn selection_to_paint(&self, focused: bool) -> Option<Bounds<Pixels>> {
        self.selection.filter(|_| focused)
    }

    fn caret_to_paint(&self, focused: bool, caret_visible: bool) -> Option<Bounds<Pixels>> {
        self.caret.filter(|_| focused && caret_visible)
    }
}

fn ui_font(tokens: &Tokens) -> Font {
    Font {
        family: tokens.fonts.ui_family.clone(),
        ..Font::default()
    }
}

fn text_runs(
    length: usize,
    marked: Option<&Range<usize>>,
    font: &Font,
    colors: &Colors,
) -> Vec<TextRun> {
    let run = |len: usize, underlined: bool| TextRun {
        len,
        font: font.clone(),
        color: colors.input_text,
        background_color: None,
        underline: underlined.then(|| UnderlineStyle {
            thickness: px(1.0),
            color: Some(colors.input_text),
            wavy: false,
        }),
        strikethrough: None,
    };
    let marked = marked.cloned().unwrap_or(length..length);
    [
        run(marked.start, false),
        run(marked.end - marked.start, true),
        run(length - marked.end, false),
    ]
    .into_iter()
    .filter(|run| run.len > 0)
    .collect()
}

impl TextField {
    fn layout_line(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &App,
    ) -> TextFieldFrame {
        let tokens = &cx.theme().tokens;
        let font = ui_font(tokens);
        let font_size = tokens.scale.medium;
        let shows_placeholder = self.state.text.is_empty();
        let (content, runs) = if shows_placeholder {
            let placeholder_run = TextRun {
                len: self.placeholder.len(),
                font: font.clone(),
                color: tokens.colors.input_placeholder,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            (self.placeholder.clone(), vec![placeholder_run])
        } else {
            (
                SharedString::from(self.text()),
                text_runs(
                    self.state.text.len(),
                    self.marked.as_ref(),
                    &font,
                    &tokens.colors,
                ),
            )
        };
        let line = window
            .text_system()
            .shape_line(content, font_size, &runs, None);
        let head = self.state.selection.main().head;
        let caret_x = if shows_placeholder {
            px(0.0)
        } else {
            line.x_for_index(head)
        };
        let content_width = if shows_placeholder {
            px(0.0)
        } else {
            line.width()
        };
        let scroll_x = revealing_scroll(
            self.geometry.scroll_x,
            caret_x,
            content_width,
            bounds.size.width,
            CARET_WIDTH,
        );
        self.geometry = LineGeometry {
            line,
            shows_placeholder,
            origin: bounds.origin,
            line_height: bounds.size.height,
            scroll_x,
        };
        let selected = self.selected_range();
        let selection = (!selected.is_empty()).then(|| {
            let start = self.geometry.x_for_offset(selected.start);
            let end = self.geometry.x_for_offset(selected.end);
            Bounds::new(
                point(start, bounds.top()),
                size(end - start, bounds.size.height),
            )
        });
        let caret = selected.is_empty().then(|| {
            Bounds::new(
                point(self.geometry.x_for_offset(head), bounds.top()),
                size(CARET_WIDTH, bounds.size.height),
            )
        });
        TextFieldFrame {
            line_origin: point(bounds.left() - scroll_x, bounds.top()),
            selection,
            caret,
        }
    }
}

impl Element for TextFieldElement {
    type RequestLayoutState = ();
    type PrepaintState = TextFieldFrame;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = cx.theme().tokens.scale.line_height.into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.field
            .update(cx, |field, cx| field.layout_line(bounds, window, cx))
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        frame: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let colors = cx.theme().tokens.colors;
        let field = self.field.read(cx);
        let focus_handle = field.focus_handle.clone();
        let caret_visible = field.caret_visible;
        let line = field.geometry.line.clone();
        let focused = focus_handle.is_focused(window);
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.field.clone()),
            cx,
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = frame.selection_to_paint(focused) {
                window.paint_quad(fill(selection, colors.focus_outline));
            }
            line.paint(
                frame.line_origin,
                bounds.size.height,
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
            if let Some(caret) = frame.caret_to_paint(focused, caret_visible) {
                window.paint_quad(fill(caret, colors.input_text));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use bourbon_ui::{Appearance, Theme};
    use gpui::{TestAppContext, size};

    use super::*;

    const CHAR_WIDTH: f32 = 8.4;

    fn frame_for(
        cx: &mut TestAppContext,
        text: &str,
        anchor: usize,
        head: usize,
    ) -> (TextFieldFrame, Bounds<Pixels>) {
        cx.update(|cx| Theme::install(Appearance::Light, cx));
        let (field, cx) = cx.add_window_view(|_, cx| TextField::new(cx));
        let text = text.to_owned();
        let bounds = Bounds::new(point(px(10.0), px(5.0)), size(px(101.0), px(20.0)));
        let frame = field.update_in(cx, |field, window, cx| {
            field.set_text(&text, cx);
            field.state.selection =
                bourbon_text::Selection::single(bourbon_text::SelectionRange::new(anchor, head));
            field.layout_line(bounds, window, cx)
        });
        (frame, bounds)
    }

    fn close(actual: Pixels, expected: f32) -> bool {
        (f32::from(actual) - expected).abs() < 0.05
    }

    #[test]
    fn unmarked_text_is_one_plain_run() {
        let tokens = Tokens::light();
        let runs = text_runs(5, None, &ui_font(&tokens), &tokens.colors);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].len, 5);
        assert_eq!(runs[0].underline, None);
        assert_eq!(runs[0].color, tokens.colors.input_text);
        assert_eq!(runs[0].font.family, tokens.fonts.ui_family);
    }

    #[test]
    fn marked_text_is_underlined_between_plain_runs() {
        let tokens = Tokens::light();
        let runs = text_runs(6, Some(&(1..4)), &ui_font(&tokens), &tokens.colors);
        let shape: Vec<_> = runs
            .iter()
            .map(|run| (run.len, run.underline.is_some()))
            .collect();
        assert_eq!(shape, [(1, false), (3, true), (2, false)]);
    }

    #[test]
    fn text_marked_to_the_end_has_no_empty_trailing_run() {
        let tokens = Tokens::light();
        let runs = text_runs(3, Some(&(0..3)), &ui_font(&tokens), &tokens.colors);
        assert_eq!(runs.len(), 1);
        assert!(runs[0].underline.is_some());
    }

    #[test]
    fn the_field_uses_the_ui_font() {
        let tokens = Tokens::light();
        assert_eq!(ui_font(&tokens).family.as_ref(), "Inter");
    }

    #[gpui::test]
    fn a_selection_spans_its_characters_and_hides_the_caret(cx: &mut TestAppContext) {
        let (frame, bounds) = frame_for(cx, "abcd", 1, 3);
        let selection = frame.selection.expect("selection");
        assert!(close(selection.origin.x, 10.0 + CHAR_WIDTH));
        assert!(close(selection.size.width, 2.0 * CHAR_WIDTH));
        assert_eq!(selection.origin.y, bounds.top());
        assert_eq!(selection.size.height, bounds.size.height);
        assert_eq!(frame.caret, None);
    }

    #[gpui::test]
    fn a_collapsed_selection_draws_only_the_caret(cx: &mut TestAppContext) {
        let (frame, _) = frame_for(cx, "abcd", 2, 2);
        assert_eq!(frame.selection, None);
        let caret = frame.caret.expect("caret");
        assert!(close(caret.origin.x, 10.0 + 2.0 * CHAR_WIDTH));
        assert_eq!(caret.size.width, CARET_WIDTH);
    }

    #[gpui::test]
    fn the_line_is_painted_shifted_by_the_scroll(cx: &mut TestAppContext) {
        let (frame, _) = frame_for(cx, &"x".repeat(20), 20, 20);
        assert!(close(
            frame.line_origin.x,
            10.0 - (20.0 * CHAR_WIDTH - 100.0)
        ));
        assert_eq!(frame.line_origin.y, px(5.0));
    }

    #[test]
    fn the_caret_paints_only_while_focused_and_in_its_visible_phase() {
        let caret = Bounds::new(point(px(1.0), px(2.0)), size(px(1.0), px(20.0)));
        let frame = TextFieldFrame {
            line_origin: point(px(0.0), px(0.0)),
            selection: Some(caret),
            caret: Some(caret),
        };
        assert_eq!(frame.caret_to_paint(true, true), Some(caret));
        assert_eq!(frame.caret_to_paint(true, false), None);
        assert_eq!(frame.caret_to_paint(false, true), None);
        assert_eq!(frame.selection_to_paint(true), Some(caret));
        assert_eq!(frame.selection_to_paint(false), None);
    }
}
