use gpui::{
    AnyWindowHandle, App, AppContext as _, Bounds, Modifiers, MouseButton, MouseDownEvent,
    MouseUpEvent, Pixels, Point, TestAppContext, VisualTestContext, Window,
};

pub(crate) trait RenderFrame {
    fn render_frame(&mut self, cx: &mut App);
}

impl RenderFrame for Window {
    fn render_frame(&mut self, cx: &mut App) {
        self.refresh();
        self.draw(cx).clear(cx);
    }
}

pub(crate) fn render(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    cx.update_window(handle, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

const DRAG_STEPS: u8 = 8;

fn selector(id: &str) -> &'static str {
    Box::leak(id.to_owned().into_boxed_str())
}

fn editor_column_bounds(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    id: &str,
) -> Option<Bounds<Pixels>> {
    cx.update_window(handle, |_, window, _| {
        gpui_kit::test::TestWindowExt::try_find(window, id.to_owned()).map(|found| found.bounds())
    })
    .unwrap()
}

pub(crate) fn bounds(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    id: &str,
) -> Option<Bounds<Pixels>> {
    render(cx, handle);
    VisualTestContext::from_window(handle, cx)
        .debug_bounds(selector(id))
        .or_else(|| editor_column_bounds(cx, handle, id))
}

pub(crate) fn is_listed(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    bounds(cx, handle, id).is_some()
}

fn require(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> Bounds<Pixels> {
    bounds(cx, handle, id).unwrap_or_else(|| panic!("no element `{id}` in the last frame"))
}

pub(crate) struct Press {
    pub(crate) button: MouseButton,
    pub(crate) modifiers: Modifiers,
    pub(crate) count: usize,
}

impl Press {
    pub(crate) const LEFT: Self = Self {
        button: MouseButton::Left,
        modifiers: Modifiers {
            control: false,
            alt: false,
            shift: false,
            platform: false,
            function: false,
        },
        count: 1,
    };
}

pub(crate) fn press_at(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    position: Point<Pixels>,
    press: &Press,
) {
    let mut visual = VisualTestContext::from_window(handle, cx);
    visual.simulate_mouse_move(position, None, press.modifiers);
    for click_count in 1..=press.count {
        visual.simulate_event(MouseDownEvent {
            button: press.button,
            position,
            modifiers: press.modifiers,
            click_count,
            first_mouse: false,
        });
        visual.simulate_event(MouseUpEvent {
            button: press.button,
            position,
            modifiers: press.modifiers,
            click_count,
        });
    }
    render(cx, handle);
}

pub(crate) fn click_with(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    id: &str,
    press: &Press,
) {
    let center = require(cx, handle, id).center();
    press_at(cx, handle, center, press);
}

pub(crate) fn click(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    click_with(cx, handle, id, &Press::LEFT);
}

pub(crate) fn click_at(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    id: &str,
    offset: Point<Pixels>,
) {
    let origin = require(cx, handle, id).origin;
    press_at(cx, handle, origin + offset, &Press::LEFT);
}

pub(crate) fn right_click(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    click_with(
        cx,
        handle,
        id,
        &Press {
            button: MouseButton::Right,
            ..Press::LEFT
        },
    );
}

pub(crate) fn hover(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    let center = require(cx, handle, id).center();
    VisualTestContext::from_window(handle, cx).simulate_mouse_move(center, None, Modifiers::none());
    render(cx, handle);
}

pub(crate) fn drag(cx: &mut TestAppContext, handle: AnyWindowHandle, from: &str, to: &str) {
    let start = require(cx, handle, from).center();
    let end = require(cx, handle, to).center();
    drag_between(cx, handle, start, end);
}

pub(crate) fn drag_by(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    id: &str,
    distance: Point<Pixels>,
) {
    let start = require(cx, handle, id).center();
    drag_between(cx, handle, start, start + distance);
}

fn drag_between(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    start: Point<Pixels>,
    end: Point<Pixels>,
) {
    let mut visual = VisualTestContext::from_window(handle, cx);
    visual.simulate_mouse_move(start, None, Modifiers::none());
    visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    for step in 1..=DRAG_STEPS {
        let point = start + (end - start) * (f32::from(step) / f32::from(DRAG_STEPS));
        visual.simulate_mouse_move(point, MouseButton::Left, Modifiers::none());
    }
    visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
    render(cx, handle);
}

pub(crate) fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, keystrokes: &str) {
    render(cx, handle);
    VisualTestContext::from_window(handle, cx).simulate_keystrokes(keystrokes);
    render(cx, handle);
}

pub(crate) fn type_text(cx: &mut TestAppContext, handle: AnyWindowHandle, text: &str) {
    render(cx, handle);
    VisualTestContext::from_window(handle, cx).simulate_input(text);
    render(cx, handle);
}
