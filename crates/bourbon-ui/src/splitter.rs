use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    App, AppContext as _, Context, DragMoveEvent, ElementId, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, Pixels, Render, RenderOnce, StatefulInteractiveElement as _,
    Styled as _, Window, div, px,
};

use crate::theme::ThemeExt;

type ResizeHandler = Rc<dyn Fn(Pixels, &mut Window, &mut App)>;
type EndHandler = Rc<dyn Fn(&mut Window, &mut App)>;

struct SplitterDrag {
    grab_offset: Rc<Cell<Pixels>>,
}

struct DragGhost;

impl Render for DragGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidthLimits {
    pub minimum: Pixels,
    pub default: Pixels,
    pub maximum: Pixels,
}

impl WidthLimits {
    #[must_use]
    pub fn clamp(self, width: Pixels) -> Pixels {
        width.max(self.minimum).min(self.maximum)
    }
}

#[must_use]
pub fn dragged_width(
    start_width: Pixels,
    start_pointer: Pixels,
    pointer: Pixels,
    limits: WidthLimits,
) -> Pixels {
    limits.clamp(start_width + (pointer - start_pointer))
}

#[derive(IntoElement)]
pub struct Splitter {
    id: ElementId,
    width: Pixels,
    limits: WidthLimits,
    on_resize: Option<ResizeHandler>,
    on_resize_end: Option<EndHandler>,
}

impl Splitter {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, width: Pixels, limits: WidthLimits) -> Self {
        Self {
            id: id.into(),
            width,
            limits,
            on_resize: None,
            on_resize_end: None,
        }
    }

    #[must_use]
    pub fn on_resize(mut self, handler: impl Fn(Pixels, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(Rc::new(handler));
        self
    }

    #[must_use]
    pub fn on_resize_end(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_resize_end = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Splitter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = &cx.theme().tokens;
        let handle = tokens.scale.splitter_handle;
        let border = tokens.scale.border;
        let (width, limits) = (self.width, self.limits);
        let grab_offset = Rc::new(Cell::new(px(0.)));
        let remembered_grab = grab_offset.clone();
        let on_resize = self.on_resize;
        let on_resize_end = self.on_resize_end;
        let released = on_resize_end.clone();
        let released_outside = on_resize_end;
        div()
            .relative()
            .flex_none()
            .h_full()
            .w(border)
            .bg(tokens.colors.border)
            .child(
                div()
                    .id(self.id)
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left((border - handle) / 2.)
                    .w(handle)
                    .cursor_col_resize()
                    .on_drag(SplitterDrag { grab_offset }, move |_, offset, _, cx| {
                        remembered_grab.set(offset.x);
                        cx.new(|_| DragGhost)
                    })
                    .on_drag_move(move |event: &DragMoveEvent<SplitterDrag>, window, cx| {
                        let grab = event.drag(cx).grab_offset.get();
                        let handle_pointer = event.bounds.origin.x + grab;
                        let resized =
                            dragged_width(width, handle_pointer, event.event.position.x, limits);
                        if let Some(on_resize) = &on_resize {
                            on_resize(resized, window, cx);
                        }
                    })
                    .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                        if let Some(released) = &released {
                            released(window, cx);
                        }
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                        if let Some(released) = &released_outside {
                            released(window, cx);
                        }
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Modifiers, TestAppContext, point, px};

    use super::*;
    use crate::test_support::host;

    const LIMITS: WidthLimits = WidthLimits {
        minimum: px(170.),
        default: px(200.),
        maximum: px(500.),
    };

    #[test]
    fn dragging_right_widens_by_the_pointer_distance() {
        assert_eq!(
            dragged_width(px(200.), px(200.), px(260.), LIMITS),
            px(260.)
        );
    }

    #[test]
    fn dragging_left_narrows_by_the_pointer_distance() {
        assert_eq!(
            dragged_width(px(300.), px(300.), px(250.), LIMITS),
            px(250.)
        );
    }

    #[test]
    fn the_width_never_goes_below_the_minimum() {
        assert_eq!(dragged_width(px(200.), px(200.), px(0.), LIMITS), px(170.));
    }

    #[test]
    fn the_width_never_goes_above_the_maximum() {
        assert_eq!(
            dragged_width(px(200.), px(200.), px(2000.), LIMITS),
            px(500.)
        );
    }

    #[test]
    fn clamp_keeps_widths_inside_the_limits_untouched() {
        assert_eq!(LIMITS.clamp(px(333.)), px(333.));
        assert_eq!(LIMITS.clamp(px(170.)), px(170.));
        assert_eq!(LIMITS.clamp(px(500.)), px(500.));
    }

    #[test]
    fn the_drag_offset_is_relative_to_where_the_press_started() {
        assert_eq!(
            dragged_width(px(200.), px(205.), px(215.), LIMITS),
            px(210.)
        );
    }

    #[gpui::test]
    fn dragging_the_handle_resizes_the_pane_clamps_it_and_reports_the_end(cx: &mut TestAppContext) {
        let width = Rc::new(Cell::new(px(200.)));
        let ended = Rc::new(Cell::new(0));
        let (shown, finished) = (width.clone(), ended.clone());
        let visual = host(cx, move || {
            let (writer, finished) = (shown.clone(), finished.clone());
            div()
                .flex()
                .size_full()
                .child(div().flex_none().h_full().w(shown.get()))
                .child(
                    Splitter::new("sidebar-splitter", shown.get(), LIMITS)
                        .on_resize(move |resized, window, _| {
                            writer.set(resized);
                            window.refresh();
                        })
                        .on_resize_end(move |_, _| finished.set(finished.get() + 1)),
                )
                .into_any_element()
        });

        visual.simulate_mouse_down(
            point(px(200.5), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        visual.simulate_mouse_move(
            point(px(204.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        visual.simulate_mouse_move(
            point(px(230.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        visual.run_until_parked();
        assert!(
            (width.get() - px(226.)).abs() < px(0.01),
            "{:?}",
            width.get()
        );
        visual.simulate_mouse_move(
            point(px(260.5), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        visual.run_until_parked();
        assert!(
            (width.get() - px(256.5)).abs() < px(0.01),
            "{:?}",
            width.get()
        );
        visual.simulate_mouse_move(
            point(px(900.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        visual.run_until_parked();
        assert_eq!(width.get(), px(500.));
        visual.simulate_mouse_up(
            point(px(900.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );

        assert_eq!(ended.get(), 1);
    }
}
