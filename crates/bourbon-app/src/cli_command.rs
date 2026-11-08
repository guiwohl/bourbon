use bourbon_core::NoteId;
use gpui::{App, ClipboardItem};

pub(crate) fn cli_command(id: &NoteId) -> String {
    format!("bourbon get {id}")
}

pub(crate) fn copy_cli_command(id: &NoteId, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(cli_command(id)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_is_bourbon_get_with_the_id() {
        let id = NoteId::parse("k7f3q2m9x4ta").unwrap();
        assert_eq!(cli_command(&id), "bourbon get k7f3q2m9x4ta");
    }
}
