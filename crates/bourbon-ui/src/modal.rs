use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement as _, IntoElement, KeyBinding,
    MouseButton, ParentElement as _, Pixels, RenderOnce, Styled as _, Window, actions, deferred,
    div,
};

use crate::theme::ThemeExt;

actions!(bourbon_modal, [Dismiss, Submit]);

pub const KEY_CONTEXT: &str = "BourbonModal";

const MODAL_PRIORITY: usize = 0;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Submit, Some(KEY_CONTEXT)),
    ]);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    Centered,
    Top(Pixels),
}

#[derive(IntoElement)]
pub struct Modal {
    id: ElementId,
    content: AnyElement,
    placement: Placement,
    focus: Option<FocusHandle>,
    on_dismiss: Option<Handler>,
    on_submit: Option<Handler>,
}

impl Modal {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            placement: Placement::Centered,
            focus: None,
            on_dismiss: None,
            on_submit: None,
        }
    }

    #[must_use]
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
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

impl RenderOnce for Modal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let backdrop = cx.theme().tokens.colors.backdrop;
        let dismiss_on_click = self.on_dismiss.clone();
        let dismiss_on_key = self.on_dismiss;
        let submit = self.on_submit;
        let surface = div()
            .key_context(KEY_CONTEXT)
            .when_some(self.focus, |surface, focus| surface.track_focus(&focus))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .when_some(dismiss_on_key, |surface, dismiss| {
                surface.on_action(move |_: &Dismiss, window, cx| dismiss(window, cx))
            })
            .when_some(submit, |surface, submit| {
                surface.on_action(move |_: &Submit, window, cx| submit(window, cx))
            })
            .child(self.content);
        let backdrop = div()
            .id(self.id)
            .occlude()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(backdrop)
            .map(|backdrop| match self.placement {
                Placement::Centered => backdrop.justify_center(),
                Placement::Top(offset) => backdrop.pt(offset),
            })
            .when_some(dismiss_on_click, |backdrop, dismiss| {
                backdrop.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    dismiss(window, cx);
                })
            })
            .child(surface);
        deferred(backdrop).with_priority(MODAL_PRIORITY)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::test_support::prepare;

    struct Counters {
        dismissed: Rc<Cell<usize>>,
        submitted: Rc<Cell<usize>>,
    }

    struct Host {
        focus: FocusHandle,
        placement: Placement,
        dismissed: Rc<Cell<usize>>,
        submitted: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let (dismissed, submitted) = (self.dismissed.clone(), self.submitted.clone());
            div().relative().size_full().child(
                Modal::new("modal", div().w(px(200.)).h(px(100.)))
                    .placement(self.placement)
                    .track_focus(&self.focus)
                    .on_dismiss(move |_, _| dismissed.set(dismissed.get() + 1))
                    .on_submit(move |_, _| submitted.set(submitted.get() + 1)),
            )
        }
    }

    fn open(cx: &mut TestAppContext, placement: Placement) -> (Counters, &mut VisualTestContext) {
        prepare(cx);
        cx.update(bind_keys);
        let dismissed = Rc::new(Cell::new(0));
        let submitted = Rc::new(Cell::new(0));
        let counters = Counters {
            dismissed: dismissed.clone(),
            submitted: submitted.clone(),
        };
        let (host, visual) = cx.add_window_view(move |_, cx| Host {
            focus: cx.focus_handle(),
            placement,
            dismissed,
            submitted,
        });
        visual.update(|window, cx| {
            let focus = host.read(cx).focus.clone();
            window.focus(&focus, cx);
        });
        visual.run_until_parked();
        (counters, visual)
    }

    fn window_center(visual: &mut VisualTestContext) -> gpui::Point<Pixels> {
        visual.update(|window, _| window.bounds().size.center())
    }

    #[gpui::test]
    fn clicking_the_backdrop_dismisses(cx: &mut TestAppContext) {
        let (counters, visual) = open(cx, Placement::Centered);

        visual.simulate_click(point(px(5.), px(5.)), Modifiers::none());

        assert_eq!(counters.dismissed.get(), 1);
    }

    #[gpui::test]
    fn clicking_the_centered_content_keeps_it_open(cx: &mut TestAppContext) {
        let (counters, visual) = open(cx, Placement::Centered);
        let center = window_center(visual);

        visual.simulate_click(center, Modifiers::none());

        assert_eq!(counters.dismissed.get(), 0);
    }

    #[gpui::test]
    fn top_placement_puts_the_content_under_the_offset(cx: &mut TestAppContext) {
        let (counters, visual) = open(cx, Placement::Top(px(56.)));
        let center = window_center(visual);

        visual.simulate_click(point(center.x, px(60.)), Modifiers::none());
        assert_eq!(counters.dismissed.get(), 0);

        visual.simulate_click(point(center.x, px(50.)), Modifiers::none());
        assert_eq!(counters.dismissed.get(), 1);
    }

    #[gpui::test]
    fn escape_dismisses_and_enter_submits(cx: &mut TestAppContext) {
        let (counters, visual) = open(cx, Placement::Centered);

        visual.simulate_keystrokes("enter");
        assert_eq!((counters.dismissed.get(), counters.submitted.get()), (0, 1));

        visual.simulate_keystrokes("escape");
        assert_eq!((counters.dismissed.get(), counters.submitted.get()), (1, 1));
    }
}
