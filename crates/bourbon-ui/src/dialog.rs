use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled as _, Window, div, px,
};

use crate::modal::Modal;
use crate::theme::ThemeExt;

const WIDTH: f32 = 420.;
const MIN_WIDTH: f32 = 240.;
const HEADER_PADDING_VERTICAL: f32 = 16.;
const HEADER_PADDING_HORIZONTAL: f32 = 17.;
const CONTENT_PADDING: f32 = 14.;
const CONTENT_GAP: f32 = 10.;
const ACTION_MIN_WIDTH: f32 = 140.;
const ACTION_GAP: f32 = 10.5;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    title: SharedString,
    content: Vec<AnyElement>,
    actions: Vec<AnyElement>,
    focus: Option<FocusHandle>,
    on_dismiss: Option<Handler>,
    on_submit: Option<Handler>,
}

impl Dialog {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: Vec::new(),
            actions: Vec::new(),
            focus: None,
            on_dismiss: None,
            on_submit: None,
        }
    }

    #[must_use]
    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.content.push(content.into_any_element());
        self
    }

    #[must_use]
    pub fn action(mut self, button: impl IntoElement) -> Self {
        self.actions.push(button.into_any_element());
        self
    }

    #[must_use]
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    #[must_use]
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    #[must_use]
    pub fn on_submit(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let header = div()
            .py(px(HEADER_PADDING_VERTICAL))
            .px(px(HEADER_PADDING_HORIZONTAL))
            .border_b_1()
            .border_color(colors.internal_border)
            .text_size(tokens.scale.medium)
            .font_weight(tokens.scale.weight_semibold)
            .child(self.title);
        let content = div()
            .flex()
            .flex_col()
            .gap(px(CONTENT_GAP))
            .p(px(CONTENT_PADDING))
            .children(self.content);
        let actions = div()
            .flex()
            .justify_end()
            .gap(px(ACTION_GAP))
            .p(px(CONTENT_PADDING))
            .border_t_1()
            .border_color(colors.internal_border)
            .bg(colors.modal_actions_background)
            .children(self.actions.into_iter().map(|button| {
                div()
                    .flex()
                    .flex_col()
                    .min_w(px(ACTION_MIN_WIDTH))
                    .child(button)
            }));
        let frame = div()
            .flex()
            .flex_col()
            .w(px(WIDTH))
            .min_w(px(MIN_WIDTH))
            .overflow_hidden()
            .rounded(tokens.scale.radius_large)
            .shadow(tokens.shadows.modal.clone())
            .bg(colors.menu_background)
            .text_color(colors.text)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.medium)
            .child(header)
            .child(content)
            .child(actions);
        let mut modal = Modal::new(self.id, frame);
        if let Some(focus) = &self.focus {
            modal = modal.track_focus(focus);
        }
        if let Some(dismiss) = self.on_dismiss {
            modal = modal.on_dismiss(move |window, cx| dismiss(window, cx));
        }
        if let Some(submit) = self.on_submit {
            modal = modal.on_submit(move |window, cx| submit(window, cx));
        }
        modal
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use gpui::{Modifiers, TestAppContext, VisualTestContext, point};

    use super::*;
    use crate::button::Button;
    use crate::test_support::host;

    fn open(cx: &mut TestAppContext, pressed: Rc<Cell<usize>>) -> &mut VisualTestContext {
        host(cx, move || {
            let pressed = pressed.clone();
            div()
                .relative()
                .size_full()
                .child(
                    Dialog::new("dialog", "Delete notebook")
                        .child("Delete it?")
                        .action(
                            Button::new("delete")
                                .label("Delete")
                                .on_click(move |_, _, _| pressed.set(pressed.get() + 1)),
                        ),
                )
                .into_any_element()
        })
    }

    #[gpui::test]
    fn the_action_buttons_sit_in_the_strip_at_the_bottom_right(cx: &mut TestAppContext) {
        let pressed = Rc::new(Cell::new(0));
        let visual = open(cx, pressed.clone());
        let size = visual.update(|window, _| window.bounds().size);
        let frame_right = (size.width + px(WIDTH)) / 2.;

        visual.simulate_click(
            point(frame_right - px(40.), size.height / 2. + px(40.)),
            Modifiers::none(),
        );

        assert_eq!(pressed.get(), 1);
    }

    #[gpui::test]
    fn the_title_area_is_not_a_button(cx: &mut TestAppContext) {
        let pressed = Rc::new(Cell::new(0));
        let visual = open(cx, pressed.clone());
        let size = visual.update(|window, _| window.bounds().size);

        visual.simulate_click(
            point(size.width / 2., size.height / 2. - px(40.)),
            Modifiers::none(),
        );

        assert_eq!(pressed.get(), 0);
    }
}
