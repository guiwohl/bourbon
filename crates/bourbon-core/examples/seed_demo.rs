use std::path::{Path, PathBuf};

use bourbon_core::{
    NoteSeed, NoteStatus, NoteStore, NotebookId, StoreError, TagColor, TagId, Timestamp,
};

struct DemoNote {
    title: &'static str,
    body: &'static str,
    status: NoteStatus,
    pinned: bool,
}

const WORK_NOTES: [DemoNote; 3] = [
    DemoNote {
        title: "Quarterly planning",
        body: "# Goals\n\n- Ship the editor preview\n- Cut startup time in half\n- [ ] Draft the roadmap\n- [x] Collect feedback",
        status: NoteStatus::Active,
        pinned: true,
    },
    DemoNote {
        title: "Meeting with design",
        body: "Talked about **typography** and spacing.\n\n> Less chrome, more content.",
        status: NoteStatus::OnHold,
        pinned: false,
    },
    DemoNote {
        title: "Old migration plan",
        body: "Replaced by the new schema work.",
        status: NoteStatus::Dropped,
        pinned: false,
    },
];

const PERSONAL_NOTES: [DemoNote; 3] = [
    DemoNote {
        title: "Reading list",
        body: "1. The Pragmatic Programmer\n2. Designing Data-Intensive Applications\n3. A Philosophy of Software Design",
        status: NoteStatus::None,
        pinned: false,
    },
    DemoNote {
        title: "Bourbon tasting",
        body: "| Bottle | Notes |\n|---|---|\n| Buffalo Trace | Vanilla, caramel |\n| Four Roses | Fruit, spice |",
        status: NoteStatus::Completed,
        pinned: false,
    },
    DemoNote {
        title: "",
        body: "A note without a title, to see the fallback.",
        status: NoteStatus::None,
        pinned: false,
    },
];

fn main() {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: seed_demo <database path>");
        std::process::exit(2);
    };
    if let Err(error) = seed(&path) {
        eprintln!("seed_demo: {error}");
        std::process::exit(1);
    }
}

fn seed(path: &Path) -> Result<(), StoreError> {
    let store = NoteStore::open(path)?;
    let now = Timestamp::now();
    let inbox = store.default_notebook()?;
    let work = store.create_notebook("Work", None, now)?.id;
    let projects = store.create_notebook("Projects", Some(&work), now)?.id;
    let personal = store.create_notebook("Personal", None, now)?.id;

    let idea = store.create_tag("idea", TagColor::Yellow, now)?.id;
    let urgent = store.create_tag("urgent", TagColor::Red, now)?.id;
    let reference = store.create_tag("reference", TagColor::Blue, now)?.id;

    write_notes(&store, &work, &WORK_NOTES, std::slice::from_ref(&urgent))?;
    write_notes(&store, &personal, &PERSONAL_NOTES, &[reference])?;
    write_notes(
        &store,
        &projects,
        &[DemoNote {
            title: "Bourbon UI ideas",
            body: "- Softer sidebar\n- Animated note switch\n- Follow the Omarchy theme",
            status: NoteStatus::Active,
            pinned: false,
        }],
        &[idea.clone(), urgent],
    )?;
    write_notes(
        &store,
        &inbox,
        &[DemoNote {
            title: "Welcome to Bourbon",
            body: "Notes save on their own half a second after you stop typing.",
            status: NoteStatus::None,
            pinned: true,
        }],
        &[idea],
    )?;

    let trashed = store.create_note(&inbox, now)?;
    store.save_note(
        &trashed.id,
        "Deleted draft",
        "This one is in the Trash.",
        now,
    )?;
    store.trash_note(&trashed.id, now)?;

    let template = store.create_template(&inbox, now)?;
    store.save_note(
        &template.id,
        "Daily log {{date}}",
        "## {{date}} {{time}}\n\n- ",
        now,
    )?;
    Ok(())
}

fn write_notes(
    store: &NoteStore,
    notebook: &NotebookId,
    notes: &[DemoNote],
    tags: &[TagId],
) -> Result<(), StoreError> {
    for demo in notes {
        let seed = NoteSeed {
            tags: tags.to_vec(),
            status: demo.status,
        };
        let note = store.create_note_with(notebook, &seed, Timestamp::now())?;
        store.save_note(&note.id, demo.title, demo.body, Timestamp::now())?;
        if demo.pinned {
            store.pin_note(&note.id)?;
        }
    }
    Ok(())
}
