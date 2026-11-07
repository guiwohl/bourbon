use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce, ScrollHandle,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::theme::ThemeExt;

const MINIMUM_THUMB_LENGTH: f32 = 24.;
const GROUP: &str = "bourbon-scroll-area";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thumb {
    pub top: f32,
    pub length: f32,
}

#[must_use]
pub fn thumb(viewport: f32, content: f32, scrolled: f32, minimum_length: f32) -> Option<Thumb> {
    if content <= viewport || viewport <= 0. {
        return None;
    }
    let length = (viewport * viewport / content)
        .max(minimum_length)
        .min(viewport);
    let scrollable = content - viewport;
    let progress = scrolled.clamp(0., scrollable) / scrollable;
    Some(Thumb {
        top: progress * (viewport - length),
        length,
    })
}

#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    handle: ScrollHandle,
    children: Vec<gpui::AnyElement>,
}

impl ScrollArea {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, handle: &ScrollHandle) -> Self {
        Self {
            id: id.into(),
            handle: handle.clone(),
            children: Vec::new(),
        }
    }
}

impl ParentElement for ScrollArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = &cx.theme().tokens;
        let viewport = self.handle.bounds().size.height.as_f32();
        let scrollable = self.handle.max_offset().y.as_f32();
        let scrolled = -self.handle.offset().y.as_f32();
        let visible = thumb(
            viewport,
            viewport + scrollable,
            scrolled,
            MINIMUM_THUMB_LENGTH,
        );
        let width = tokens.scale.scrollbar_width;
        let hover_width = tokens.scale.scrollbar_hover_width;
        let thumb_color = tokens.colors.scrollbar_thumb;
        let track_color = tokens.colors.scrollbar_track;
        div()
            .group(GROUP)
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .id(self.id)
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.handle)
                    .children(self.children),
            )
            .children(visible.map(|visible| {
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .w(width)
                    .bg(track_color)
                    .group_hover(GROUP, move |style| style.w(hover_width))
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(visible.top))
                            .h(px(visible.length))
                            .rounded(width / 2.)
                            .bg(thumb_color),
                    )
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_that_fits_has_no_thumb() {
        assert_eq!(thumb(400., 400., 0., 20.), None);
        assert_eq!(thumb(400., 100., 0., 20.), None);
    }

    #[test]
    fn the_thumb_length_is_the_visible_share_of_the_track() {
        let at_top = thumb(100., 400., 0., 10.).unwrap();
        assert!((at_top.length - 25.).abs() < 1e-4);
        assert!(at_top.top.abs() < 1e-4);
    }

    #[test]
    fn scrolling_to_the_end_puts_the_thumb_at_the_bottom() {
        let at_end = thumb(100., 400., 300., 10.).unwrap();
        assert!((at_end.top + at_end.length - 100.).abs() < 1e-4);
    }

    #[test]
    fn halfway_scrolled_puts_the_thumb_halfway() {
        let middle = thumb(100., 400., 150., 10.).unwrap();
        let free_track = 100. - middle.length;
        assert!((middle.top - free_track / 2.).abs() < 1e-4);
    }

    #[test]
    fn a_tiny_thumb_is_stretched_to_the_minimum_length() {
        let stretched = thumb(100., 100_000., 0., 24.).unwrap();
        assert!((stretched.length - 24.).abs() < 1e-4);
    }

    #[test]
    fn overscroll_is_held_inside_the_track() {
        let past_end = thumb(100., 400., 900., 10.).unwrap();
        assert!((past_end.top + past_end.length - 100.).abs() < 1e-4);
        let before_start = thumb(100., 400., -50., 10.).unwrap();
        assert!(before_start.top.abs() < 1e-4);
    }
}
