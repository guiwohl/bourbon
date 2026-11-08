mod change;
pub mod commands;
mod compose;
mod fold;
mod history;
mod markup;
pub mod movement;
mod search;
mod selection;
mod state;
mod text;

#[cfg(test)]
mod testing;

pub use change::{Assoc, Change, ChangeSet};
pub use fold::Folds;
pub use history::History;
pub use search::{InvalidPattern, Search, SearchQuery, current_match, next_match, previous_match};
pub use selection::{Selection, SelectionRange};
pub use state::{EditorState, Transaction, TransactionKind};
pub use text::Text;
