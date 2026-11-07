use gpui::{App, IntoElement, RenderOnce, Styled as _, Window, div, px};

use crate::theme::ThemeExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(IntoElement)]
pub struct Divider {
    orientation: Orientation,
}

impl Divider {
    #[must_use]
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }

    #[must_use]
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
        }
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = &cx.theme().tokens;
        let line = div().flex_none().bg(tokens.colors.internal_border);
        match self.orientation {
            Orientation::Horizontal => line.w_full().h(tokens.scale.border),
            Orientation::Vertical => line.h_full().w(px(1.)),
        }
    }
}
