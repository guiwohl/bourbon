use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    Anchor, AnyElement, App, Bounds, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Point, RenderOnce, Window, anchored, deferred, div, point, px,
};

const GAP_BELOW_ANCHOR: f32 = 4.;
const WINDOW_MARGIN: f32 = 8.;
const POPOVER_PRIORITY: usize = 1;

type DismissHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[must_use]
pub fn below(bounds: Bounds<Pixels>) -> Point<Pixels> {
    bounds.bottom_left() + point(px(0.), px(GAP_BELOW_ANCHOR))
}

#[derive(IntoElement)]
pub struct Popover {
    position: Point<Pixels>,
    content: AnyElement,
    on_dismiss: Option<DismissHandler>,
}

impl Popover {
    #[must_use]
    pub fn at(position: Point<Pixels>, content: impl IntoElement) -> Self {
        Self {
            position,
            content: content.into_any_element(),
            on_dismiss: None,
        }
    }

    #[must_use]
    pub fn below(bounds: Bounds<Pixels>, content: impl IntoElement) -> Self {
        Self::at(below(bounds), content)
    }

    #[must_use]
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let dismiss = self.on_dismiss;
        let surface = div()
            .occlude()
            .when_some(dismiss, |surface, dismiss| {
                surface.on_mouse_down_out(move |_, window, cx| dismiss(window, cx))
            })
            .child(self.content);
        deferred(
            anchored()
                .position(self.position)
                .anchor(Anchor::TopLeft)
                .snap_to_window_with_margin(px(WINDOW_MARGIN))
                .child(surface),
        )
        .with_priority(POPOVER_PRIORITY)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, Styled as _, TestAppContext, size};

    use super::*;
    use crate::test_support::host;

    #[test]
    fn below_sits_under_the_bottom_left_corner_with_a_small_gap() {
        let bounds = Bounds {
            origin: point(px(10.), px(20.)),
            size: size(px(100.), px(30.)),
        };
        assert_eq!(below(bounds), point(px(10.), px(54.)));
    }

    fn dismissal_counter(
        cx: &mut TestAppContext,
    ) -> (Rc<Cell<usize>>, &mut gpui::VisualTestContext) {
        let dismissed = Rc::new(Cell::new(0));
        let counter = dismissed.clone();
        let visual = host(cx, move || {
            let counter = counter.clone();
            Popover::at(point(px(100.), px(100.)), div().w(px(100.)).h(px(50.)))
                .on_dismiss(move |_, _| counter.set(counter.get() + 1))
                .into_any_element()
        });
        (dismissed, visual)
    }

    #[gpui::test]
    fn clicking_outside_the_popover_dismisses_it(cx: &mut TestAppContext) {
        let (dismissed, visual) = dismissal_counter(cx);

        visual.simulate_click(point(px(700.), px(500.)), Modifiers::none());

        assert_eq!(dismissed.get(), 1);
    }

    #[gpui::test]
    fn clicking_inside_the_popover_keeps_it_open(cx: &mut TestAppContext) {
        let (dismissed, visual) = dismissal_counter(cx);

        visual.simulate_click(point(px(150.), px(120.)), Modifiers::none());

        assert_eq!(dismissed.get(), 0);
    }
}
