use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::icon::{Icon, IconName, Stroke};
use crate::theme::ThemeExt;

const BOX_SIZE: f32 = 17.;
const BOX_RADIUS: f32 = 6.;
const CHECK_SIZE: f32 = 12.;
const CHECK_OPACITY: f32 = 0.9;
const LABEL_GAP: f32 = 9.;

type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    label: SharedString,
    description: Option<SharedString>,
    disabled: bool,
    on_toggle: Option<ToggleHandler>,
}

impl Checkbox {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            label: label.into(),
            description: None,
            disabled: false,
            on_toggle: None,
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let checked = self.checked;
        let mark = div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .size(px(BOX_SIZE))
            .rounded(px(BOX_RADIUS))
            .border_1()
            .map(|mark| {
                if checked {
                    mark.bg(colors.primary).border_color(colors.primary).child(
                        div().opacity(CHECK_OPACITY).child(
                            Icon::new(IconName::Check)
                                .size(px(CHECK_SIZE))
                                .stroke(Stroke::Heavy)
                                .color(colors.inverted_text),
                        ),
                    )
                } else {
                    mark.bg(colors.input_background)
                        .border_color(colors.strong_border)
                }
            });
        let line = div()
            .flex()
            .items_center()
            .gap(px(LABEL_GAP))
            .child(mark)
            .child(self.label);
        let root = div()
            .id(self.id)
            .flex()
            .flex_col()
            .text_size(tokens.scale.medium)
            .text_color(colors.text)
            .child(line)
            .when_some(self.description, |root, description| {
                root.child(
                    div()
                        .pl(px(BOX_SIZE + LABEL_GAP))
                        .text_size(tokens.scale.small)
                        .text_color(colors.muted_text)
                        .child(description),
                )
            });
        if self.disabled {
            return root.opacity(tokens.scale.disabled_opacity);
        }
        root.cursor_pointer()
            .when_some(self.on_toggle, |root, toggle| {
                root.on_click(move |_, window, cx| toggle(!checked, window, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, point};

    use super::*;
    use crate::test_support::host;

    fn toggles(cx: &mut TestAppContext, checked: bool, disabled: bool) -> Vec<bool> {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let recorded = seen.clone();
        let visual = host(cx, move || {
            let recorded = recorded.clone();
            Checkbox::new("backup", "Automatic backup", checked)
                .disabled(disabled)
                .on_toggle(move |value, _, _| recorded.borrow_mut().push(value))
                .into_any_element()
        });
        visual.simulate_click(point(px(40.), px(8.)), Modifiers::none());
        seen.take()
    }

    #[gpui::test]
    fn clicking_an_unchecked_box_asks_to_check_it(cx: &mut TestAppContext) {
        assert_eq!(toggles(cx, false, false), [true]);
    }

    #[gpui::test]
    fn clicking_a_checked_box_asks_to_uncheck_it(cx: &mut TestAppContext) {
        assert_eq!(toggles(cx, true, false), [false]);
    }

    #[gpui::test]
    fn a_disabled_box_ignores_clicks(cx: &mut TestAppContext) {
        assert_eq!(toggles(cx, false, true), Vec::<bool>::new());
    }
}
