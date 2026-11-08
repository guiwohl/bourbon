use gpui::{App, KeyBinding, actions};

pub const SEARCH_CONTEXT: &str = "BourbonSearch";

actions!(
    bourbon_search,
    [
        FindNext,
        FindPrevious,
        ToggleMatchCase,
        ToggleRegex,
        ToggleWholeWord,
        ReplaceAll,
    ]
);

pub fn bind_search_keys(cx: &mut App) {
    let context = Some(SEARCH_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("shift-enter", FindPrevious, context),
        KeyBinding::new("ctrl-g", FindNext, context),
        KeyBinding::new("ctrl-shift-g", FindPrevious, context),
        KeyBinding::new("alt-c", ToggleMatchCase, context),
        KeyBinding::new("alt-r", ToggleRegex, context),
        KeyBinding::new("alt-w", ToggleWholeWord, context),
        KeyBinding::new("ctrl-enter", ReplaceAll, context),
    ]);
}
