mod change;
pub mod commands;
mod compose;
mod history;
mod markup;
pub mod movement;
mod selection;
mod state;
mod text;

#[cfg(test)]
mod testing;

pub use change::{Assoc, Change, ChangeSet};
pub use history::History;
pub use selection::{Selection, SelectionRange};
pub use state::{EditorState, Transaction, TransactionKind};
pub use text::Text;
