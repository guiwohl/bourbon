use gpui::{
    Context, CursorStyle, Div, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
    div,
};

use crate::actions::{self, KEY_CONTEXT};
use crate::editor::Editor;
use crate::element::EditorElement;
use crate::motion::Motion;

pub(crate) fn editor_root(editor: &Editor, cx: &mut Context<Editor>) -> impl IntoElement {
    let root = div()
        .key_context(KEY_CONTEXT)
        .track_focus(&editor.focus_handle)
        .size_full()
        .overflow_hidden()
        .cursor(CursorStyle::IBeam)
        .bg(editor.theme.background)
        .on_mouse_down(MouseButton::Left, cx.listener(Editor::on_mouse_down))
        .on_mouse_move(cx.listener(Editor::on_mouse_move))
        .on_mouse_up(MouseButton::Left, cx.listener(Editor::on_mouse_up))
        .on_mouse_up_out(MouseButton::Left, cx.listener(Editor::on_mouse_up))
        .on_scroll(cx.listener(Editor::on_scroll));
    with_motion_actions(with_edit_actions(root, cx), cx).child(EditorElement::new(cx.entity()))
}

fn with_motion_actions(root: Div, cx: &mut Context<Editor>) -> Div {
    root.on_action(
        cx.listener(|e, _: &actions::Left, _, cx| e.move_horizontal(Motion::Left, false, cx)),
    )
    .on_action(
        cx.listener(|e, _: &actions::Right, _, cx| e.move_horizontal(Motion::Right, false, cx)),
    )
    .on_action(
        cx.listener(|e, _: &actions::SelectLeft, _, cx| e.move_horizontal(Motion::Left, true, cx)),
    )
    .on_action(cx.listener(|e, _: &actions::SelectRight, _, cx| {
        e.move_horizontal(Motion::Right, true, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::WordLeft, _, cx| {
        e.move_horizontal(Motion::WordLeft, false, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::WordRight, _, cx| {
        e.move_horizontal(Motion::WordRight, false, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectWordLeft, _, cx| {
        e.move_horizontal(Motion::WordLeft, true, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectWordRight, _, cx| {
        e.move_horizontal(Motion::WordRight, true, cx);
    }))
    .on_action(
        cx.listener(|e, _: &actions::Home, _, cx| e.move_horizontal(Motion::LineStart, false, cx)),
    )
    .on_action(
        cx.listener(|e, _: &actions::End, _, cx| e.move_horizontal(Motion::LineEnd, false, cx)),
    )
    .on_action(cx.listener(|e, _: &actions::SelectHome, _, cx| {
        e.move_horizontal(Motion::LineStart, true, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectEnd, _, cx| {
        e.move_horizontal(Motion::LineEnd, true, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::DocumentStart, _, cx| {
        e.move_horizontal(Motion::DocumentStart, false, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::DocumentEnd, _, cx| {
        e.move_horizontal(Motion::DocumentEnd, false, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectDocumentStart, _, cx| {
        e.move_horizontal(Motion::DocumentStart, true, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectDocumentEnd, _, cx| {
        e.move_horizontal(Motion::DocumentEnd, true, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::Up, w, cx| e.move_vertical(-1, false, w, cx)))
    .on_action(cx.listener(|e, _: &actions::Down, w, cx| e.move_vertical(1, false, w, cx)))
    .on_action(cx.listener(|e, _: &actions::SelectUp, w, cx| e.move_vertical(-1, true, w, cx)))
    .on_action(cx.listener(|e, _: &actions::SelectDown, w, cx| e.move_vertical(1, true, w, cx)))
    .on_action(cx.listener(|e, _: &actions::PageUp, w, cx| {
        let rows = e.page_rows();
        e.move_vertical(-rows, false, w, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::PageDown, w, cx| {
        let rows = e.page_rows();
        e.move_vertical(rows, false, w, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectPageUp, w, cx| {
        let rows = e.page_rows();
        e.move_vertical(-rows, true, w, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectPageDown, w, cx| {
        let rows = e.page_rows();
        e.move_vertical(rows, true, w, cx);
    }))
    .on_action(cx.listener(|e, _: &actions::SelectAll, _, cx| e.select_all(cx)))
}

fn with_edit_actions(root: Div, cx: &mut Context<Editor>) -> Div {
    root.on_action(cx.listener(|e, _: &actions::Backspace, _, cx| e.backspace(cx)))
        .on_action(cx.listener(|e, _: &actions::Delete, _, cx| e.delete_forward(cx)))
        .on_action(
            cx.listener(|e, _: &actions::DeleteWordBackward, _, cx| e.delete_word_backward(cx)),
        )
        .on_action(
            cx.listener(|e, _: &actions::DeleteWordForward, _, cx| e.delete_word_forward(cx)),
        )
        .on_action(cx.listener(|e, _: &actions::Enter, _, cx| e.enter(cx)))
        .on_action(cx.listener(|e, _: &actions::Tab, _, cx| e.indent(cx)))
        .on_action(cx.listener(|e, _: &actions::ShiftTab, _, cx| e.outdent(cx)))
        .on_action(cx.listener(|e, _: &actions::Undo, _, cx| e.undo(cx)))
        .on_action(cx.listener(|e, _: &actions::Redo, _, cx| e.redo(cx)))
        .on_action(cx.listener(|e, _: &actions::Copy, _, cx| {
            e.copy(cx);
        }))
        .on_action(cx.listener(|e, _: &actions::Cut, _, cx| e.cut(cx)))
        .on_action(cx.listener(|e, _: &actions::Paste, _, cx| e.paste(cx)))
        .on_action(cx.listener(|e, _: &actions::ToggleBold, _, cx| e.toggle_bold(cx)))
        .on_action(cx.listener(|e, _: &actions::ToggleItalic, _, cx| e.toggle_italic(cx)))
        .on_action(
            cx.listener(|e, _: &actions::ToggleStrikethrough, _, cx| e.toggle_strikethrough(cx)),
        )
        .on_action(cx.listener(|e, _: &actions::ToggleCode, _, cx| e.toggle_code(cx)))
        .on_action(cx.listener(|e, _: &actions::ToggleTask, _, cx| e.toggle_task(cx)))
        .on_action(cx.listener(|e, _: &actions::Fold, _, cx| e.fold_at_caret(cx)))
        .on_action(cx.listener(|e, _: &actions::Unfold, _, cx| e.unfold_at_caret(cx)))
}
