use gpui::{App, KeyBinding};

use crate::actions::{
    Backspace, Cancel, Copy, Cut, Delete, DeleteWordBackward, DeleteWordForward, End, Enter, Home,
    Left, Paste, Redo, Right, SelectAll, SelectEnd, SelectHome, SelectLeft, SelectRight,
    SelectWordLeft, SelectWordRight, Undo, WordLeft, WordRight,
};

pub const TEXT_FIELD_CONTEXT: &str = "BourbonTextField";

pub fn bind_text_field_keys(cx: &mut App) {
    let context = Some(TEXT_FIELD_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("ctrl-left", WordLeft, context),
        KeyBinding::new("ctrl-right", WordRight, context),
        KeyBinding::new("ctrl-shift-left", SelectWordLeft, context),
        KeyBinding::new("ctrl-shift-right", SelectWordRight, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-home", SelectHome, context),
        KeyBinding::new("shift-end", SelectEnd, context),
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("ctrl-backspace", DeleteWordBackward, context),
        KeyBinding::new("ctrl-delete", DeleteWordForward, context),
        KeyBinding::new("enter", Enter, context),
        KeyBinding::new("escape", Cancel, context),
        KeyBinding::new("ctrl-z", Undo, context),
        KeyBinding::new("ctrl-shift-z", Redo, context),
        KeyBinding::new("ctrl-y", Redo, context),
        KeyBinding::new("ctrl-a", SelectAll, context),
        KeyBinding::new("ctrl-c", Copy, context),
        KeyBinding::new("ctrl-x", Cut, context),
        KeyBinding::new("ctrl-v", Paste, context),
    ]);
}
