use gpui::{
    App, BorderStyle, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window, fill, point,
    px, quad, relative, size,
};

use crate::editor::Editor;
use crate::frame::Frame;

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
        self.editor
            .update(cx, |editor, _| editor.build_frame(bounds, window))
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
                window.paint_quad(fill(
                    Bounds::new(
                        point(bounds.left(), line.origin.y),
                        size(bounds.size.width, line.height),
                    ),
                    theme.active_line,
                ));
            }
            if frame.gutter_width > px(0.0) {
                window.paint_quad(fill(
                    Bounds::new(
                        point(bounds.left() + frame.gutter_width - px(1.0), bounds.top()),
                        size(px(1.0), bounds.size.height),
                    ),
                    theme.gutter_border,
                ));
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
