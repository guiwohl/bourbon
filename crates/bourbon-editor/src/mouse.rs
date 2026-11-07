use gpui::{
    Context, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollEvent,
    Window, point, px,
};

use bourbon_text::SelectionRange;

use crate::editor::{Editor, EditorEvent};
use crate::motion::{extend_to, line_range_at, word_range_at};
use crate::task_click::{task_marker_at, toggle_task_at};

pub(crate) struct HitTest {
    pub offset: usize,
    pub glyph: Option<usize>,
}

impl Editor {
    pub(crate) fn hit_test(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
    ) -> Option<HitTest> {
        let bounds = self.viewport?;
        let y = position.y.max(bounds.top()).min(bounds.bottom() - px(1.0));
        let document_y = f32::from(y - bounds.top() + self.scroll_top);
        let line = self.heights.line_at(document_y);
        let layout = self.layout_for_line(line, window);
        let local = point(
            position.x - bounds.left() - self.text_left,
            px(document_y - self.heights.top(line)),
        );
        let line_start = self.state.text.line_start(line);
        Some(HitTest {
            offset: line_start + layout.index_for_position(local),
            glyph: layout.glyph_index_at(local).map(|index| line_start + index),
        })
    }

    pub(crate) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        let Some(hit) = self.hit_test(event.position, window) else {
            return;
        };
        if event.click_count == 1 && !event.modifiers.shift && self.toggle_task_under(&hit, cx) {
            return;
        }
        let main = self.state.selection.main();
        let selected = match event.click_count {
            1 if event.modifiers.shift => extend_to(main, hit.offset),
            1 => SelectionRange::cursor(hit.offset),
            2 => word_range_at(&self.state.text, hit.offset),
            _ => line_range_at(&self.state.text, hit.offset),
        };
        self.select(selected, cx);
        self.dragging = true;
    }

    fn toggle_task_under(&mut self, hit: &HitTest, cx: &mut Context<Self>) -> bool {
        let Some(glyph) = hit.glyph else {
            return false;
        };
        if task_marker_at(&self.spans, glyph).is_none() {
            return false;
        }
        let line = self.state.text.line_of(glyph);
        if !self.run_command(|state| toggle_task_at(state, glyph), cx) {
            return false;
        }
        cx.emit(EditorEvent::TaskToggled { line });
        true
    }

    pub(crate) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.dragging || event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        if let Some(hit) = self.hit_test(event.position, window) {
            let moved = extend_to(self.state.selection.main(), hit.offset);
            self.select_keeping_goal(moved, cx);
        }
    }

    pub(crate) fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.dragging = false;
    }

    pub(crate) fn on_scroll(
        &mut self,
        event: &ScrollEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delta = event.delta.pixel_delta(self.line_height());
        self.scroll_by(-delta.y, cx);
    }
}
