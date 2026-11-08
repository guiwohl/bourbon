use std::ops::Range;

use bourbon_text::commands::insert_text;
use bourbon_text::{EditorState, Selection, SelectionRange};
use gpui::{
    Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point, px, size,
};

use super::{TextField, single_line};
use crate::utf16::{offset_to_utf16, range_from_utf16, range_to_utf16};

impl TextField {
    fn edit_target(&self, range_utf16: Option<&Range<usize>>) -> Range<usize> {
        let text = self.text();
        range_utf16
            .map(|range| range_from_utf16(&text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selected_range())
    }

    fn insert_over(&mut self, target: &Range<usize>, text: &str, cx: &mut Context<Self>) {
        let probe = EditorState::new(
            self.state.text.clone(),
            Selection::single(SelectionRange::new(target.start, target.end)),
        );
        if let Some(transaction) = insert_text(&probe, text) {
            self.apply(&transaction, cx);
        }
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.text();
        let range = range_from_utf16(&text, &range_utf16);
        adjusted_range.replace(range_to_utf16(&text, &range));
        Some(text[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let main = self.state.selection.main();
        Some(UTF16Selection {
            range: range_to_utf16(&self.text(), &(main.from()..main.to())),
            reversed: main.is_reversed(),
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        let text = self.text();
        self.marked
            .as_ref()
            .map(|range| range_to_utf16(&text, range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = self.edit_target(range_utf16.as_ref());
        self.insert_over(&target, &single_line(new_text), cx);
        self.marked = None;
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = self.edit_target(range_utf16.as_ref());
        let new_text = single_line(new_text);
        self.insert_over(&target, &new_text, cx);
        let inserted = target.start..target.start + new_text.len();
        let selected = new_selected_range_utf16.map_or(inserted.end..inserted.end, |range| {
            let relative = range_from_utf16(&new_text, &range);
            inserted.start + relative.start..inserted.start + relative.end
        });
        self.state.selection = Selection::single(SelectionRange::new(selected.start, selected.end));
        self.marked = (!new_text.is_empty()).then_some(inserted);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = range_from_utf16(&self.text(), &range_utf16);
        let start = self.geometry.x_for_offset(range.start);
        let end = self.geometry.x_for_offset(range.end);
        Some(Bounds::new(
            point(start, self.geometry.origin.y),
            size((end - start).max(px(1.0)), self.geometry.line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let offset = self.geometry.offset_for_x(position.x);
        Some(offset_to_utf16(&self.text(), offset))
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_from_utf16(&self.text(), &range_utf16);
        self.select(SelectionRange::new(range.start, range.end), cx);
    }

    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let text = self.text();
        Some(offset_to_utf16(&text, text.len()))
    }
}
