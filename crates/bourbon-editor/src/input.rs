use std::ops::Range;

use bourbon_text::commands::{insert_text, type_character};
use bourbon_text::{EditorState, Selection, SelectionRange};
use gpui::{
    Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point, px, size,
};

use crate::editor::Editor;
use crate::utf16::{offset_to_utf16, range_from_utf16, range_to_utf16};

impl Editor {
    fn edit_target(&self, range_utf16: Option<&Range<usize>>) -> Range<usize> {
        let text = self.text();
        range_utf16
            .map(|range| range_from_utf16(&text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| {
                let selected = self.state.selection.main();
                selected.from()..selected.to()
            })
    }

    fn state_selecting(&self, range: &Range<usize>) -> EditorState {
        EditorState::new(
            self.state.text.clone(),
            Selection::single(SelectionRange::new(range.start, range.end)),
        )
    }
}

impl EntityInputHandler for Editor {
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
        let text = self.text();
        let main = self.state.selection.main();
        Some(UTF16Selection {
            range: range_to_utf16(&text, &(main.from()..main.to())),
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
        self.invalidate_layouts();
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let single_typed_character = range_utf16.is_none() && self.marked.is_none();
        let target = self.edit_target(range_utf16.as_ref());
        let probe = self.state_selecting(&target);
        let mut characters = new_text.chars();
        let transaction = match (characters.next(), characters.next()) {
            (Some(character), None) if single_typed_character => type_character(&probe, character),
            _ => insert_text(&probe, new_text),
        };
        if let Some(transaction) = transaction {
            self.apply(&transaction, cx);
        }
        self.marked = None;
        self.invalidate_layouts();
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
        let probe = self.state_selecting(&target);
        if let Some(transaction) = insert_text(&probe, new_text) {
            self.apply(&transaction, cx);
        }
        let inserted = target.start..target.start + new_text.len();
        let selected = new_selected_range_utf16.map_or(inserted.end..inserted.end, |range| {
            let relative = range_from_utf16(new_text, &range);
            inserted.start + relative.start..inserted.start + relative.end
        });
        self.state.selection = Selection::single(SelectionRange::new(selected.start, selected.end));
        self.marked = (!new_text.is_empty()).then_some(inserted);
        self.invalidate_layouts();
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let text = self.text();
        let range = range_from_utf16(&text, &range_utf16);
        let line = self.state.text.line_of(range.start);
        let line_start = self.state.text.line_start(line);
        let layout = self.layout_for_line(line, window);
        let start = layout.position_for_index(range.start - line_start);
        let end =
            layout.position_for_index(range.end.min(self.state.text.line_end(line)) - line_start);
        let width = if end.y == start.y {
            (end.x - start.x).max(px(1.0))
        } else {
            px(1.0)
        };
        let origin = point(
            element_bounds.left() + self.text_left + start.x,
            element_bounds.top() + px(self.heights.top(line)) - self.scroll_top + start.y,
        );
        Some(Bounds::new(
            origin,
            size(width, layout.row_height_at(range.start - line_start)),
        ))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let hit = self.hit_test(position, window)?;
        let text = self.text();
        Some(offset_to_utf16(&text, hit.offset))
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.text();
        let range = range_from_utf16(&text, &range_utf16);
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
