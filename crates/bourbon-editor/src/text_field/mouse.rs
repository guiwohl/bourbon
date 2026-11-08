use bourbon_text::SelectionRange;
use gpui::{Context, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Window};

use super::TextField;
use crate::motion::{extend_to, word_range_at};

impl TextField {
    pub(super) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        let offset = self.geometry.offset_for_x(event.position.x);
        let main = self.state.selection.main();
        let selected = match event.click_count {
            1 if event.modifiers.shift => extend_to(main, offset),
            1 => SelectionRange::cursor(offset),
            2 => word_range_at(&self.state.text, offset),
            _ => SelectionRange::new(0, self.state.text.len()),
        };
        self.select(selected, cx);
        self.dragging = true;
    }

    pub(super) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.dragging || event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let offset = self.geometry.offset_for_x(event.position.x);
        let moved = extend_to(self.state.selection.main(), offset);
        self.select(moved, cx);
    }

    pub(super) fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.dragging = false;
    }
}
