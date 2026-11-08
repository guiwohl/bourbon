use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
};

use crate::icon::{Icon, IconName};
use crate::theme::ThemeExt;
use crate::tooltip::Tooltip;

fn control_width(segment: gpui::Pixels, count: f32, border: gpui::Pixels) -> gpui::Pixels {
    segment * count + border * 2.
}

fn is_selected(index: usize, selected: usize) -> bool {
    index == selected
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

pub struct Segment {
    pub icon: IconName,
    pub label: SharedString,
}

impl Segment {
    #[must_use]
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            icon,
            label: label.into(),
        }
    }
}

#[derive(IntoElement)]
pub struct SegmentedControl {
    id: SharedString,
    segments: Vec<Segment>,
    selected: usize,
    on_change: Option<ChangeHandler>,
}

impl SegmentedControl {
    #[must_use]
    pub fn new(id: impl Into<SharedString>, segments: Vec<Segment>) -> Self {
        Self {
            id: id.into(),
            segments,
            selected: 0,
            on_change: None,
        }
    }

    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let selected = self.selected;
        let on_change = self.on_change;
        let count = f32::from(u8::try_from(self.segments.len()).unwrap_or(u8::MAX));
        let segments = self
            .segments
            .into_iter()
            .enumerate()
            .map(|(index, segment)| {
                let is_selected = is_selected(index, selected);
                let handler = on_change.clone();
                let label = segment.label;
                let mut button = div()
                    .id(ElementId::from((self.id.clone(), index)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(tokens.scale.segment_width)
                    .h(tokens.scale.segment_height)
                    .cursor_pointer()
                    .child(
                        Icon::new(segment.icon)
                            .size(tokens.scale.segment_icon)
                            .color(if is_selected {
                                colors.primary
                            } else {
                                colors.muted_text
                            }),
                    )
                    .tooltip(move |_, cx| Tooltip::text(label.clone(), cx));
                button = if is_selected {
                    button.bg(colors.button_active_background)
                } else {
                    let hover = colors.button_hover_background;
                    button.hover(move |style| style.bg(hover))
                };
                match handler {
                    Some(handler) => {
                        button.on_click(move |_, window, cx| handler(index, window, cx))
                    }
                    None => button,
                }
            });
        let width = control_width(tokens.scale.segment_width, count, tokens.scale.border);
        div()
            .flex()
            .flex_none()
            .w(width)
            .overflow_hidden()
            .border_1()
            .border_color(colors.border)
            .rounded(tokens.scale.radius_control)
            .bg(colors.menu_background)
            .children(segments)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, point, px};

    use super::*;
    use crate::test_support::host;

    fn view_modes(
        selected: usize,
        changes: Rc<RefCell<Vec<usize>>>,
    ) -> impl Fn() -> gpui::AnyElement {
        move || {
            let changes = changes.clone();
            SegmentedControl::new(
                "view-mode",
                vec![
                    Segment::new(IconName::Pencil, "Show Editor"),
                    Segment::new(IconName::Columns2, "View Side by Side"),
                    Segment::new(IconName::Eye, "Show Preview"),
                ],
            )
            .selected(selected)
            .on_change(move |index, _, _| changes.borrow_mut().push(index))
            .into_any_element()
        }
    }

    fn segment_centre(index: u8) -> gpui::Point<gpui::Pixels> {
        point(px(1. + 44. * f32::from(index) + 22.), px(15.))
    }

    #[test]
    fn the_control_is_as_wide_as_its_segments_plus_both_borders() {
        assert_eq!(control_width(px(44.), 3., px(1.)), px(134.));
        assert_eq!(control_width(px(44.), 1., px(1.)), px(46.));
    }

    #[test]
    fn only_the_matching_index_is_selected() {
        assert!(is_selected(1, 1));
        assert!(!is_selected(0, 1));
    }

    #[gpui::test]
    fn clicking_a_segment_reports_its_index(cx: &mut TestAppContext) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let visual = host(cx, view_modes(0, changes.clone()));

        visual.simulate_click(segment_centre(2), Modifiers::none());
        visual.simulate_click(segment_centre(1), Modifiers::none());

        assert_eq!(*changes.borrow(), [2, 1]);
    }

    #[gpui::test]
    fn clicking_the_segment_already_selected_still_reports_it(cx: &mut TestAppContext) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let visual = host(cx, view_modes(1, changes.clone()));

        visual.simulate_click(segment_centre(1), Modifiers::none());

        assert_eq!(*changes.borrow(), [1]);
    }
}
