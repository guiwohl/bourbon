use gpui::{
    App, BorderStyle, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window, fill, point,
    px, quad, relative, size,
};

use crate::editor::Editor;
use crate::frame::{Frame, FrameLine};
use crate::theme::EditorTheme;

pub fn active_line_bounds(bounds: Bounds<Pixels>, line: &FrameLine) -> Bounds<Pixels> {
    Bounds::new(
        point(bounds.left(), line.origin.y),
        size(bounds.size.width, line.height),
    )
}

pub fn gutter_border_bounds(
    bounds: Bounds<Pixels>,
    gutter_width: Pixels,
) -> Option<Bounds<Pixels>> {
    (gutter_width > px(0.0)).then(|| {
        Bounds::new(
            point(bounds.left() + gutter_width - px(1.0), bounds.top()),
            size(px(1.0), bounds.size.height),
        )
    })
}

fn paint_decorations(line: &FrameLine, theme: &EditorTheme, window: &mut Window) {
    if let Some(card) = &line.card {
        window.paint_quad(quad(
            card.bounds,
            card.radii,
            theme.code_background,
            card.borders,
            theme.code_border,
            BorderStyle::default(),
        ));
    }
    if let Some(bar) = line.quote_bar {
        window.paint_quad(fill(bar, theme.quote_bar));
    }
    let radius = theme.font_size * theme.card_radius_em;
    for code_box in &line.code_boxes {
        window.paint_quad(quad(
            *code_box,
            radius,
            theme.code_background,
            px(1.0),
            theme.code_border,
            BorderStyle::default(),
        ));
    }
}

pub struct EditorElement {
    editor: Entity<Editor>,
}

impl EditorElement {
    pub fn new(editor: Entity<Editor>) -> Self {
        Self { editor }
    }
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = Frame;

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
        style.size.height = relative(1.0).into();
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
        let frame = self.editor.update(cx, |editor, cx| {
            let frame = editor.build_frame(bounds, window);
            editor.report_scroll(cx);
            frame
        });
        if frame.measuring_pending {
            window.request_animation_frame();
        }
        frame
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
        let (focus_handle, theme, caret_visible) = {
            let editor = self.editor.read(cx);
            (
                editor.focus_handle.clone(),
                editor.theme.clone(),
                editor.caret_visible,
            )
        };
        let focused = focus_handle.is_focused(window);
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for line in frame.lines.iter().filter(|line| line.active) {
                window.paint_quad(fill(active_line_bounds(bounds, line), theme.active_line));
            }
            for line in &frame.lines {
                paint_decorations(line, &theme, window);
            }
            if let Some(border) = gutter_border_bounds(bounds, frame.gutter_width) {
                window.paint_quad(fill(border, theme.gutter_border));
            }
            let selection_color = if focused {
                theme.selection
            } else {
                theme.selection_unfocused
            };
            for rect in frame.lines.iter().flat_map(|line| line.selections.iter()) {
                window.paint_quad(fill(*rect, selection_color));
            }
            for line in &frame.lines {
                line.layout.paint(line.origin, window, cx);
                if let Some((number, origin)) = line.number.as_ref() {
                    number
                        .paint(
                            *origin,
                            line.layout.line_height(),
                            gpui::TextAlign::Left,
                            None,
                            window,
                            cx,
                        )
                        .ok();
                }
            }
            if let Some(caret) = frame.caret
                && focused
                && caret_visible
            {
                window.paint_quad(quad(
                    caret,
                    px(1.0),
                    theme.caret,
                    0.0,
                    gpui::transparent_black(),
                    BorderStyle::default(),
                ));
            }
        });
        self.editor.update(cx, |editor, _| editor.focused = focused);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::LineLayout;
    use std::rc::Rc;

    fn bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(100.0), px(50.0)), size(px(400.0), px(300.0)))
    }

    #[test]
    fn the_gutter_border_is_a_one_pixel_line_at_the_gutter_edge() {
        let border = gutter_border_bounds(bounds(), px(30.0)).expect("border");
        assert_eq!(border.origin, point(px(129.0), px(50.0)));
        assert_eq!(border.size, size(px(1.0), px(300.0)));
    }

    #[test]
    fn there_is_no_border_without_a_gutter() {
        assert_eq!(gutter_border_bounds(bounds(), px(0.0)), None);
    }

    #[gpui::test]
    fn the_active_line_spans_the_whole_width(cx: &mut gpui::TestAppContext) {
        let window = cx.add_empty_window();
        let layout = window.update(|window, _| {
            let theme = crate::theme::EditorTheme::light();
            LineLayout::shape(
                &crate::layout::ShapeRequest {
                    text: "",
                    segments: &[],
                    prefix_len: 0,
                    font_size: px(13.0),
                    line_height: px(19.5),
                    wrap_width: None,
                    bold: false,
                    box_model: crate::layout::LineBox::default(),
                    theme: &theme,
                },
                window,
            )
        });
        let line = FrameLine {
            origin: point(px(116.0), px(80.0)),
            height: px(39.0),
            layout: Rc::new(layout),
            active: true,
            selections: Vec::new(),
            number: None,
            card: None,
            quote_bar: None,
            code_boxes: Vec::new(),
        };
        let active = active_line_bounds(bounds(), &line);
        assert_eq!(active.origin, point(px(100.0), px(80.0)));
        assert_eq!(active.size, size(px(400.0), px(39.0)));
    }
}
