mod highlights;
mod keys;
mod panel;
mod view;

pub(crate) use highlights::{MatchRect, match_quad};
pub use keys::{
    FindNext, FindPrevious, ReplaceAll, SEARCH_CONTEXT, ToggleMatchCase, ToggleRegex,
    ToggleWholeWord, bind_search_keys,
};
pub use panel::{SearchOptions, SearchPanel, match_count_label};
