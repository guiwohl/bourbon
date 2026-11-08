mod assets;
mod autosave;
mod backup_runner;
mod cli_command;
mod control_pane;
mod dragged_note;
mod editor;
mod fonts;
mod locations;
mod navigation;
mod note_list;
mod overlay;
mod presentation;
mod settings_view;
mod sidebar;
mod single_instance;
mod test_id;
mod theme;
mod window;
mod workspace;

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use bourbon_core::{NoteStore, default_database_path};

use crate::single_instance::{Claim, claim, socket_path};

fn claim_instance() -> Claim {
    let user_id = std::fs::metadata("/proc/self").map_or(0, |metadata| metadata.uid());
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    let socket = socket_path(window::app_id(), runtime_dir, user_id);
    claim(&socket).unwrap_or_else(|error| {
        eprintln!("bourbon: cannot reach the single-instance socket: {error}");
        std::process::exit(1);
    })
}

fn main() {
    let Claim::Primary(listener) = claim_instance() else {
        return;
    };
    let database_path = default_database_path();
    let store = NoteStore::open(&database_path).unwrap_or_else(|error| {
        eprintln!(
            "bourbon: cannot open the database at {}: {error}",
            database_path.display()
        );
        std::process::exit(1);
    });
    window::run(
        store,
        locations::Locations::of_the_user(),
        listener.into_focus_requests(),
    );
}
