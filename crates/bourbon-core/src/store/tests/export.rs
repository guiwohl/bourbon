use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::support::{add_tag, at, attach_tag, inbox, store, titled_note, trash};
use crate::export::ExportSummary;
use crate::{NoteStore, StoreError, TagColor};

fn files_under(root: &Path) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let relative = path.strip_prefix(root).unwrap().display().to_string();
        if path.is_dir() {
            files.insert(format!("{relative}/"), String::new());
            collect(root, &path, files);
        } else {
            files.insert(relative, fs::read_to_string(&path).unwrap());
        }
    }
}

fn populated_store() -> NoteStore {
    let store = store();
    let inbox = inbox(&store);
    let work = store.create_notebook("Work", Some(&inbox), at(10)).unwrap();
    let kept = titled_note(&store, &inbox, "Shopping List", 1_000, 2_000);
    store
        .save_note(&kept, "Shopping List", "- milk\n- eggs\n", at(2_000))
        .unwrap();
    let urgent = add_tag(&store, "urgent", TagColor::Red);
    attach_tag(&store, &kept, &urgent);
    titled_note(&store, &work.id, "Plan: Q4 \"big\"", 3_000, 4_000);
    let gone = titled_note(&store, &inbox, "Deleted", 5_000, 5_000);
    trash(&store, &gone, 6_000);
    let template = titled_note(&store, &inbox, "Weekly", 7_000, 7_000);
    store.mark_as_template(&template).unwrap();
    store
}

#[test]
fn exports_the_exact_tree_and_contents() {
    let store = populated_store();
    let notes = store
        .exportable_notes()
        .unwrap()
        .into_iter()
        .map(|note| (note.title.clone(), note.id.to_string()))
        .collect::<BTreeMap<_, _>>();
    let sandbox = tempfile::tempdir().unwrap();
    let destination = sandbox.path().join("out");

    let summary = store.export_markdown(&destination).unwrap();

    assert_eq!(
        summary,
        ExportSummary {
            notebooks: 2,
            notes: 2,
            templates: 1
        }
    );
    let expected = BTreeMap::from([
        ("Inbox/".to_owned(), String::new()),
        (
            "Inbox/shopping-list.md".to_owned(),
            format!(
                "---\nid: {}\ntitle: \"Shopping List\"\ntags:\n  - \"urgent\"\ncreated: 1970-01-01T00:00:01Z\nupdated: 1970-01-01T00:00:02Z\n---\n\n- milk\n- eggs\n",
                notes["Shopping List"]
            ),
        ),
        ("Inbox/Work/".to_owned(), String::new()),
        (
            "Inbox/Work/plan-q4-big.md".to_owned(),
            format!(
                "---\nid: {}\ntitle: \"Plan: Q4 \\\"big\\\"\"\ntags: []\ncreated: 1970-01-01T00:00:03Z\nupdated: 1970-01-01T00:00:04Z\n---\n\n",
                notes["Plan: Q4 \"big\""]
            ),
        ),
        ("_templates/".to_owned(), String::new()),
        (
            "_templates/weekly.md".to_owned(),
            format!(
                "---\nid: {}\ntitle: \"Weekly\"\ntags: []\ncreated: 1970-01-01T00:00:07Z\nupdated: 1970-01-01T00:00:07Z\n---\n\n",
                notes["Weekly"]
            ),
        ),
    ]);
    assert_eq!(files_under(&destination), expected);
}

#[test]
fn trashed_notes_are_not_exported() {
    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    store.export_markdown(&sandbox.path().join("out")).unwrap();
    assert!(!files_under(&sandbox.path().join("out")).contains_key("Inbox/deleted.md"));
}

#[test]
fn an_empty_existing_directory_is_accepted() {
    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    store.export_markdown(sandbox.path()).unwrap();
    assert!(sandbox.path().join("Inbox/shopping-list.md").is_file());
}

#[test]
fn missing_parent_directories_are_created() {
    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    let destination = sandbox.path().join("deep/er/out");
    store.export_markdown(&destination).unwrap();
    assert!(destination.join("Inbox").is_dir());
}

#[test]
fn a_non_empty_destination_is_refused_and_left_untouched() {
    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    fs::write(sandbox.path().join("keep.txt"), "mine").unwrap();
    let error = store.export_markdown(sandbox.path()).unwrap_err();
    assert!(matches!(error, StoreError::ExportDestinationNotEmpty(path) if path == sandbox.path()));
    assert_eq!(
        files_under(sandbox.path()),
        BTreeMap::from([("keep.txt".to_owned(), "mine".to_owned())])
    );
}

#[test]
fn a_file_as_destination_is_refused() {
    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    let file = sandbox.path().join("file");
    fs::write(&file, "x").unwrap();
    let error = store.export_markdown(&file).unwrap_err();
    assert!(matches!(error, StoreError::ExportDestinationNotEmpty(_)));
}

#[test]
fn a_successful_export_leaves_no_staging_folder_behind() {
    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    store.export_markdown(&sandbox.path().join("out")).unwrap();
    let entries: Vec<_> = fs::read_dir(sandbox.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries, ["out"]);
}

#[cfg(unix)]
#[test]
fn a_failed_export_leaves_nothing_behind() {
    use std::os::unix::fs::PermissionsExt;

    let store = populated_store();
    let sandbox = tempfile::tempdir().unwrap();
    let locked = sandbox.path().join("locked");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o500)).unwrap();

    let result = store.export_markdown(&locked.join("out"));

    fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(result, Err(StoreError::Io(_))));
    assert_eq!(fs::read_dir(&locked).unwrap().count(), 0);
}
