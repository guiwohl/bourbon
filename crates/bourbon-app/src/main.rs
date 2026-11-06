mod autosave;
mod editor;
mod navigation;
mod note_list;
mod presentation;
mod sidebar;
mod window;
mod workspace;

use bourbon_core::{NoteStore, default_database_path};

fn main() {
    let database_path = default_database_path();
    let store = NoteStore::open(&database_path).unwrap_or_else(|error| {
        eprintln!(
            "bourbon: cannot open the database at {}: {error}",
            database_path.display()
        );
        std::process::exit(1);
    });
    window::run(store);
}
