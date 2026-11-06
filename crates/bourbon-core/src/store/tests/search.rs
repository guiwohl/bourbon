use super::support::{add_tag, at, attach_tag, inbox, store, titled_note, trash};
use crate::{NoteId, NoteStore, TagColor};

fn written(store: &NoteStore, title: &str, body: &str, updated: i64) -> NoteId {
    let note = store.create_note(&inbox(store), at(1)).unwrap();
    store.save_note(&note.id, title, body, at(updated)).unwrap();
    note.id
}

fn found(store: &NoteStore, text: &str) -> Vec<NoteId> {
    store
        .search_notes(text, 50)
        .unwrap()
        .into_iter()
        .map(|hit| hit.summary.id)
        .collect()
}

#[test]
fn a_word_in_the_body_finds_the_note() {
    let store = store();
    let note = written(&store, "Groceries", "buy fresh bread tomorrow", 10);
    written(&store, "Other", "nothing relevant", 20);
    assert_eq!(found(&store, "bread"), vec![note]);
}

#[test]
fn a_word_in_the_title_finds_the_note() {
    let store = store();
    let note = written(&store, "Quarterly roadmap", "", 10);
    assert_eq!(found(&store, "roadmap"), vec![note]);
}

#[test]
fn the_last_token_matches_by_prefix() {
    let store = store();
    let note = written(&store, "Architecture", "", 10);
    assert_eq!(found(&store, "archi"), vec![note]);
}

#[test]
fn every_token_must_match() {
    let store = store();
    let both = written(&store, "Plan", "alpha beta", 10);
    written(&store, "Plan two", "alpha only", 20);
    assert_eq!(found(&store, "alpha beta"), vec![both]);
}

#[test]
fn matching_ignores_case() {
    let store = store();
    let note = written(&store, "Rust", "Ownership", 10);
    assert_eq!(found(&store, "OWNERSHIP rUsT"), vec![note]);
}

#[test]
fn a_title_match_ranks_above_a_body_match() {
    let store = store();
    let in_body = written(&store, "Misc", "this mentions kiwi once", 20);
    let in_title = written(&store, "Kiwi", "", 10);
    assert_eq!(found(&store, "kiwi"), vec![in_title, in_body]);
}

#[test]
fn trashed_notes_are_not_found() {
    let store = store();
    let note = written(&store, "Secret", "hidden", 10);
    trash(&store, &note, 100);
    assert_eq!(found(&store, "secret").len(), 0);
}

#[test]
fn templates_are_not_found() {
    let store = store();
    let template = store.create_template(&inbox(&store), at(1)).unwrap();
    store
        .save_note(&template.id, "Meeting template", "agenda", at(2))
        .unwrap();
    assert_eq!(found(&store, "agenda").len(), 0);
}

#[test]
fn edits_are_searchable_and_old_text_is_not() {
    let store = store();
    let note = written(&store, "Draft", "old words", 10);
    store
        .save_note(&note, "Draft", "new words", at(20))
        .unwrap();
    assert_eq!(found(&store, "new"), vec![note]);
    assert_eq!(found(&store, "old").len(), 0);
}

#[test]
fn a_purged_note_is_not_found() {
    let store = store();
    let note = written(&store, "Gone", "soon", 10);
    store.trash_note(&note, at(20)).unwrap();
    store.empty_trash().unwrap();
    assert_eq!(found(&store, "soon").len(), 0);
}

#[test]
fn a_note_in_a_hidden_status_is_still_found() {
    let store = store();
    let note = written(&store, "Finished", "done thing", 10);
    store
        .set_status(&note, crate::NoteStatus::Completed, at(11))
        .unwrap();
    assert_eq!(found(&store, "done"), vec![note]);
}

#[test]
fn pinning_a_note_keeps_it_searchable() {
    let store = store();
    let note = written(&store, "Pinned", "keep", 10);
    store.toggle_pin(&note).unwrap();
    assert_eq!(found(&store, "keep"), vec![note]);
}

#[test]
fn odd_input_never_fails() {
    let store = store();
    written(&store, "Quote", "say \"hello\" (world) *", 10);
    for text in [
        "\"", "*", "AND", "OR", "(", ")", "NOT x", "a\"b", "col:x", "-", "^",
    ] {
        assert!(store.search_notes(text, 50).is_ok(), "failed on {text:?}");
    }
}

#[test]
fn a_quoted_token_is_matched_literally() {
    let store = store();
    let note = written(&store, "Literal", "AND is a word", 10);
    assert_eq!(found(&store, "AND"), vec![note]);
}

#[test]
fn blank_text_lists_the_most_recently_updated_first() {
    let store = store();
    let older = written(&store, "Older", "", 10);
    let newer = written(&store, "Newer", "", 20);
    assert_eq!(found(&store, ""), vec![newer.clone(), older.clone()]);
    assert_eq!(found(&store, "  \t "), vec![newer, older]);
}

#[test]
fn blank_text_skips_trashed_notes_and_templates() {
    let store = store();
    let kept = written(&store, "Kept", "", 10);
    let trashed = written(&store, "Trashed", "", 20);
    trash(&store, &trashed, 30);
    store.create_template(&inbox(&store), at(40)).unwrap();
    assert_eq!(found(&store, ""), vec![kept]);
}

#[test]
fn the_limit_caps_the_results() {
    let store = store();
    for index in 0..5 {
        written(&store, &format!("note {index}"), "common", index);
    }
    assert_eq!(store.search_notes("common", 3).unwrap().len(), 3);
    assert_eq!(store.search_notes("", 2).unwrap().len(), 2);
}

#[test]
fn a_hit_carries_the_summary_with_tags() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "Tagged", 5, 6);
    let tag = add_tag(&store, "work", TagColor::Blue);
    attach_tag(&store, &note, &tag);
    let hit = store.search_notes("tagged", 50).unwrap().remove(0);
    assert_eq!(hit.summary.id, note);
    assert_eq!(hit.summary.title, "Tagged");
    assert_eq!(hit.summary.created_at, at(5));
    assert_eq!(hit.summary.updated_at, at(6));
    assert_eq!(hit.summary.tags.len(), 1);
}

#[test]
fn the_snippet_shows_the_matching_text_without_markup() {
    let store = store();
    written(&store, "Title", "intro line\nthe needle sits here\nend", 10);
    let hit = store.search_notes("needle", 50).unwrap().remove(0);
    assert!(hit.snippet.contains("needle"), "{}", hit.snippet);
    assert!(!hit.snippet.contains('<'), "{}", hit.snippet);
    assert!(!hit.snippet.contains(['\u{1}', '\u{2}']), "{}", hit.snippet);
    assert!(!hit.snippet.contains('\n'), "{}", hit.snippet);
}

#[test]
fn the_snippet_of_a_title_only_match_is_the_excerpt() {
    let store = store();
    written(&store, "Findme", "first line\nsecond", 10);
    let hit = store.search_notes("findme", 50).unwrap().remove(0);
    assert_eq!(hit.snippet, "first line");
}

#[test]
fn blank_text_snippet_is_the_excerpt() {
    let store = store();
    written(&store, "T", "\n  hello there \nmore", 10);
    let hit = store.search_notes("", 50).unwrap().remove(0);
    assert_eq!(hit.snippet, "hello there");
}

#[test]
fn notes_present_before_the_migration_are_back_filled() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    {
        let store = NoteStore::open(&path).unwrap();
        written(&store, "Old", "ancient wisdom", 10);
        store
            .connection
            .execute_batch(
                "DROP TRIGGER notes_fts_insert; DROP TRIGGER notes_fts_update; DROP TRIGGER notes_fts_delete;
                 DROP TABLE notes_fts; PRAGMA user_version = 3;",
            )
            .unwrap();
    }
    let store = NoteStore::open(&path).unwrap();
    assert_eq!(found(&store, "ancient").len(), 1);
}
