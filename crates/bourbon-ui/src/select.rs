use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    App, Bounds, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::icon::{Icon, IconName};
use crate::theme::ThemeExt;

const PADDING_VERTICAL: f32 = 8.;
const PADDING_HORIZONTAL: f32 = 14.;
const MIN_WIDTH: f32 = 180.;
const CHEVRON_SIZE: f32 = 10.;
const CHEVRON_GAP: f32 = 12.;

type OpenHandler = Rc<dyn Fn(Bounds<Pixels>, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    on_open: Option<OpenHandler>,
}

impl Select {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            on_open: None,
        }
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn on_open(
        mut self,
        handler: impl Fn(Bounds<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Select {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let bounds = Rc::new(Cell::new(Bounds::default()));
        let mut trigger = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(CHEVRON_GAP))
            .min_w(px(MIN_WIDTH))
            .py(px(PADDING_VERTICAL))
            .px(px(PADDING_HORIZONTAL))
            .rounded(tokens.scale.radius_control)
            .border_1()
            .border_color(colors.border)
            .bg(colors.input_background)
            .text_color(colors.input_text)
            .text_size(tokens.scale.medium)
            .child(self.label)
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(px(CHEVRON_SIZE))
                    .color(colors.muted_text),
            );
        if self.disabled {
            trigger = trigger.opacity(tokens.scale.disabled_opacity);
        } else if let Some(open) = self.on_open {
            let measured = bounds.clone();
            trigger = trigger
                .cursor_pointer()
                .on_click(move |_, window, cx| open(measured.get(), window, cx));
        }
        div()
            .flex()
            .on_children_prepainted(move |children, _, _| {
                if let Some(first) = children.first() {
                    bounds.set(*first);
                }
            })
            .child(trigger)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, point};

    use super::*;
    use crate::test_support::host;

    fn opened(cx: &mut TestAppContext, disabled: bool) -> Vec<Bounds<Pixels>> {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let recorded = seen.clone();
        let visual = host(cx, move || {
            let recorded = recorded.clone();
            div()
                .pt(px(30.))
                .pl(px(20.))
                .child(
                    Select::new("theme", "Bourbon")
                        .disabled(disabled)
                        .on_open(move |bounds, _, _| recorded.borrow_mut().push(bounds)),
                )
                .into_any_element()
        });
        visual.simulate_click(point(px(40.), px(45.)), Modifiers::none());
        seen.take()
    }

    #[gpui::test]
    fn clicking_reports_the_trigger_bounds(cx: &mut TestAppContext) {
        let opened = opened(cx, false);

        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0].origin, point(px(20.), px(30.)));
        assert!(opened[0].size.width >= px(MIN_WIDTH));
    }

    #[gpui::test]
    fn a_disabled_select_does_not_open(cx: &mut TestAppContext) {
        assert_eq!(opened(cx, true), Vec::new());
    }
}
