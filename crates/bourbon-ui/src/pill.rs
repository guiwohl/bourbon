use gpui::{
    App, Hsla, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div,
    px,
};

use crate::readable::{PILL_TINT, pill_text_color};
use crate::theme::ThemeExt;

const BORDER_ALPHA: f32 = 0.3;

#[derive(IntoElement)]
pub struct Pill {
    label: SharedString,
    color: Hsla,
}

impl Pill {
    #[must_use]
    pub fn new(label: impl Into<SharedString>, color: Hsla) -> Self {
        Self {
            label: label.into(),
            color,
        }
    }
}

impl RenderOnce for Pill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let tokens = &theme.tokens;
        let text = pill_text_color(self.color, tokens.colors.page_background, theme.is_dark());
        div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(22.))
            .px(px(6.))
            .rounded(tokens.scale.radius_tag)
            .border_1()
            .border_color(self.color.alpha(BORDER_ALPHA))
            .bg(self.color.alpha(PILL_TINT))
            .text_color(text)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.tiny)
            .child(self.label)
    }
}
