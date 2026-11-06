mod autosave;
mod control_pane;
mod dragged_note;
mod editor;
mod locations;
mod navigation;
mod note_list;
mod panes;
mod presentation;
mod settings_view;
mod sidebar;
mod theme;
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
    window::run(store, locations::Locations::of_the_user());
}
