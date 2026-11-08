use gpui::prelude::FluentBuilder as _;
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
    compact: bool,
}

struct PillMetrics {
    height: Option<f32>,
    horizontal_padding: f32,
    vertical_padding: f32,
    text_size: f32,
}

impl PillMetrics {
    fn of(compact: bool) -> Self {
        if compact {
            Self {
                height: None,
                horizontal_padding: 4.,
                vertical_padding: 2.,
                text_size: 10.,
            }
        } else {
            Self {
                height: Some(22.),
                horizontal_padding: 6.,
                vertical_padding: 0.,
                text_size: 12.,
            }
        }
    }
}

impl Pill {
    #[must_use]
    pub fn new(label: impl Into<SharedString>, color: Hsla) -> Self {
        Self {
            label: label.into(),
            color,
            compact: false,
        }
    }

    #[must_use]
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

impl RenderOnce for Pill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let tokens = &theme.tokens;
        let text = pill_text_color(self.color, tokens.colors.page_background, theme.is_dark());
        let metrics = PillMetrics::of(self.compact);
        div()
            .flex()
            .flex_none()
            .items_center()
            .when_some(metrics.height, |pill, height| pill.h(px(height)))
            .px(px(metrics.horizontal_padding))
            .py(px(metrics.vertical_padding))
            .rounded(tokens.scale.radius_tag)
            .border_1()
            .border_color(self.color.alpha(BORDER_ALPHA))
            .bg(self.color.alpha(PILL_TINT))
            .text_color(text)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(px(metrics.text_size))
            .child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_compact_pill_is_smaller_than_the_regular_one() {
        let regular = PillMetrics::of(false);
        let compact = PillMetrics::of(true);

        assert!(compact.text_size < regular.text_size);
        assert!(compact.horizontal_padding < regular.horizontal_padding);
        assert!(regular.height.is_some());
        assert!(compact.height.is_none());
    }

    #[test]
    fn a_pill_is_regular_unless_asked_to_be_compact() {
        let pill = Pill::new("rust", gpui::red());
        assert!(!pill.compact);
        assert!(pill.compact().compact);
    }
}
