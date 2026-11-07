use bourbon_text::commands::{
    delete_backward, delete_forward, delete_word_backward, indent_list_item,
    insert_newline_continue_markup, insert_text, outdent_list_item, toggle_bold,
    toggle_inline_code, toggle_italic, toggle_strikethrough, toggle_task,
};
use bourbon_text::movement::next_word_boundary;
use bourbon_text::{EditorState, Selection, SelectionRange, Transaction, TransactionKind};
use gpui::{ClipboardItem, Context, Pixels, Window, point, px};

use crate::editor::{Editor, line_height_of};
use crate::motion::{Motion, extend_to, move_range, walk_rows};
use crate::numeric::{count_f32, row_from_ratio, signed};
use crate::task_click::toggle_task_at;

const INDENT: &str = "  ";

impl Editor {
    pub(crate) fn move_horizontal(&mut self, motion: Motion, extend: bool, cx: &mut Context<Self>) {
        let moved = move_range(
            &self.state.text,
            self.state.selection.main(),
            motion,
            extend,
        );
        self.select(moved, cx);
    }

    pub(crate) fn move_vertical(
        &mut self,
        rows: i64,
        extend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let main = self.state.selection.main();
        let line = self.state.text.line_of(main.head);
        let line_start = self.state.text.line_start(line);
        let layout = self.layout_for_line(line, window);
        let local = layout.position_for_index(main.head - line_start);
        let goal_x = self.goal_x.unwrap_or(local.x);
        let row = row_from_ratio(local.y / layout.line_height());
        let line_count = self.state.text.line_count();
        let target = walk_rows(line, row, rows, line_count, |candidate| {
            self.layout_for_line(candidate, window).row_count()
        });
        let offset = if target == (line, row) {
            if rows < 0 { 0 } else { self.state.text.len() }
        } else {
            let target_layout = self.layout_for_line(target.0, window);
            let height = target_layout.line_height();
            let probe = point(goal_x, height * count_f32(target.1) + height * 0.5);
            self.state.text.line_start(target.0) + target_layout.index_for_position(probe)
        };
        let moved = if extend {
            extend_to(main, offset)
        } else {
            SelectionRange::cursor(offset)
        };
        self.select_keeping_goal(moved, cx);
        self.goal_x = Some(goal_x);
    }

    pub(crate) fn page_rows(&self) -> i64 {
        let viewport = self.viewport.map_or(px(0.0), |bounds| bounds.size.height);
        let line_height = line_height_of(&self.theme, self.theme.font_size);
        (signed(row_from_ratio(viewport / line_height)) - 1).max(1)
    }

    pub(crate) fn select_all(&mut self, cx: &mut Context<Self>) {
        let length = self.state.text.len();
        self.select(SelectionRange::new(0, length), cx);
    }

    pub(crate) fn backspace(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_backward, cx);
    }

    pub(crate) fn delete_forward(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_forward, cx);
    }

    pub(crate) fn delete_word_backward(&mut self, cx: &mut Context<Self>) {
        self.run_command(delete_word_backward, cx);
    }

    pub(crate) fn delete_word_forward(&mut self, cx: &mut Context<Self>) {
        self.run_command(
            |state| {
                let head = state.selection.main().head;
                let end = next_word_boundary(&state.text, head);
                let probe = EditorState::new(
                    state.text.clone(),
                    Selection::single(SelectionRange::new(head, end)),
                );
                delete_forward(&probe)
            },
            cx,
        );
    }

    pub(crate) fn enter(&mut self, cx: &mut Context<Self>) {
        let continued = self.run_command(insert_newline_continue_markup, cx);
        if !continued {
            self.run_command(|state| insert_text(state, "\n"), cx);
        }
    }

    pub(crate) fn indent(&mut self, cx: &mut Context<Self>) {
        let indented = self.run_command(indent_list_item, cx);
        if !indented {
            self.run_command(|state| insert_text(state, INDENT), cx);
        }
    }

    pub(crate) fn outdent(&mut self, cx: &mut Context<Self>) {
        self.run_command(outdent_list_item, cx);
    }

    pub(crate) fn toggle_bold(&mut self, cx: &mut Context<Self>) {
        self.run_command(toggle_bold, cx);
    }

    pub(crate) fn toggle_italic(&mut self, cx: &mut Context<Self>) {
        self.run_command(toggle_italic, cx);
    }

    pub(crate) fn toggle_strikethrough(&mut self, cx: &mut Context<Self>) {
        self.run_command(toggle_strikethrough, cx);
    }

    pub(crate) fn toggle_code(&mut self, cx: &mut Context<Self>) {
        self.run_command(toggle_inline_code, cx);
    }

    pub(crate) fn toggle_task(&mut self, cx: &mut Context<Self>) {
        self.run_command(toggle_task, cx);
    }

    pub fn toggle_task_at_line(&mut self, line: usize, cx: &mut Context<Self>) {
        if line >= self.state.text.line_count() {
            return;
        }
        let offset = self.state.text.line_start(line);
        self.run_command(|state| toggle_task_at(state, offset), cx);
    }

    pub(crate) fn copy(&mut self, cx: &mut Context<Self>) -> bool {
        let selected = self.state.selection.main();
        if selected.is_empty() {
            return false;
        }
        let text = self.state.text.slice(selected.from()..selected.to());
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        true
    }

    pub(crate) fn cut(&mut self, cx: &mut Context<Self>) {
        if self.copy(cx) {
            self.run_command(delete_backward, cx);
        }
    }

    pub(crate) fn paste(&mut self, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let normalized = text.replace("\r\n", "\n");
        self.run_command(
            |state| {
                insert_text(state, &normalized).map(|transaction| Transaction {
                    kind: TransactionKind::Paste,
                    isolate: true,
                    ..transaction
                })
            },
            cx,
        );
    }

    pub(crate) fn line_height(&self) -> Pixels {
        line_height_of(&self.theme, self.theme.font_size)
    }
}
