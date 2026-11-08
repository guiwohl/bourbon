use bourbon_ui::{Colors, ThemeExt as _};
use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, Font,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, SharedString, Style,
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
    selection: Option<Bounds<Pixels>>,
    caret: Option<Bounds<Pixels>>,
}

impl TextField {
    fn text_runs(&self, font: &Font, colors: &Colors) -> Vec<TextRun> {
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
        let length = self.state.text.len();
        let marked = self.marked.clone().unwrap_or(length..length);
        [
            run(marked.start, false),
            run(marked.end - marked.start, true),
            run(length - marked.end, false),
        ]
        .into_iter()
        .filter(|run| run.len > 0)
        .collect()
    }

    fn layout_line(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &App,
    ) -> TextFieldFrame {
        let tokens = &cx.theme().tokens;
        let font = Font {
            family: tokens.fonts.ui_family.clone(),
            ..Font::default()
        };
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
                self.text_runs(&font, &tokens.colors),
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
        TextFieldFrame { selection, caret }
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
        let line_origin = point(bounds.left() - field.geometry.scroll_x, bounds.top());
        let focused = focus_handle.is_focused(window);
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.field.clone()),
            cx,
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = frame.selection.filter(|_| focused) {
                window.paint_quad(fill(selection, colors.focus_outline));
            }
            line.paint(
                line_origin,
                bounds.size.height,
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
            if let Some(caret) = frame.caret.filter(|_| focused && caret_visible) {
                window.paint_quad(fill(caret, colors.input_text));
            }
        });
    }
}
