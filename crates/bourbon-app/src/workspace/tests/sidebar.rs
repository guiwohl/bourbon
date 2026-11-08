use bourbon_core::{Note, NoteId, NoteStatus, NoteStore, Notebook, NotebookDepth, Tag, TagColor};
use gpui::{
    AnyWindowHandle, AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext,
};
use gpui::{point, px};

use super::harness::{Press, RenderFrame as _, click_with};
use super::{
    at, banner_is_shown, bounds, choose_menu_entry, click, click_at, is_listed, listed_titles,
    menu_labels, note_in, open_view, press, render, right_click, type_text,
};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

const NEW_SUB_NOTEBOOK: usize = 0;
const RENAME_NOTEBOOK: usize = 1;
const DELETE_NOTEBOOK: usize = 3;
const RENAME_TAG: usize = 0;
const CHANGE_COLOUR: usize = 1;
const DELETE_TAG: usize = 2;

struct Tree {
    store: NoteStore,
    work: Notebook,
    sub: Notebook,
    other: Notebook,
}

fn tree_store() -> Tree {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let sub = store.create_notebook("Sub", Some(&work.id), at(2)).unwrap();
    let other = store.create_notebook("Other", None, at(2)).unwrap();
    note_in(&store, &inbox, "In inbox", 3);
    note_in(&store, &work.id, "In work", 4);
    note_in(&store, &sub.id, "In sub", 5);
    Tree {
        store,
        work,
        sub,
        other,
    }
}

fn green_position() -> usize {
    TagColor::ALL
        .iter()
        .position(|color| *color == TagColor::Green)
        .unwrap()
}

fn row(notebook: &Notebook) -> String {
    format!("nav-notebook-{}", notebook.id)
}

fn tag_row(tag: &Tag) -> String {
    format!("nav-tag-{}", tag.id)
}

fn ctrl_click(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    click_with(
        cx,
        handle,
        id,
        &Press {
            modifiers: Modifiers::control(),
            ..Press::LEFT
        },
    );
}

fn open_menu(cx: &mut TestAppContext, handle: AnyWindowHandle, row: &str) {
    right_click(cx, handle, row);
    cx.run_until_parked();
}

fn choose_menu_item(cx: &mut TestAppContext, handle: AnyWindowHandle, row: &str, index: usize) {
    open_menu(cx, handle, row);
    choose_menu_entry(cx, handle, index);
}

fn menu_labels_of(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    row: &str,
) -> Vec<String> {
    open_menu(cx, handle, row);
    menu_labels(cx, view)
}

fn toggle_notebook(cx: &mut TestAppContext, handle: AnyWindowHandle, notebook: &Notebook) {
    click_at(cx, handle, &row(notebook), point(px(20.), px(15.)));
}

fn click_section_collapse(cx: &mut TestAppContext, handle: AnyWindowHandle, header: &str) {
    let width = bounds(cx, handle, header).unwrap().size.width;
    click_at(cx, handle, header, point(width - px(17.), px(15.)));
}

fn type_name(cx: &mut TestAppContext, handle: AnyWindowHandle, name: &str) {
    press(cx, handle, "ctrl-a");
    type_text(cx, handle, name);
}

fn navigation(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Navigation {
    view.read_with(cx, |view, _| view.listing.navigation.clone())
}

fn notebook_names(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<String> {
    view.read_with(cx, |view, _| {
        fn collect(nodes: &[bourbon_core::NotebookNode], names: &mut Vec<String>) {
            for node in nodes {
                names.push(node.notebook.name.clone());
                collect(&node.children, names);
            }
        }
        let mut names = Vec::new();
        collect(&view.notebooks, &mut names);
        names
    })
}

fn stored_tag(cx: &mut TestAppContext, view: &Entity<Workspace>, name: &str) -> Option<Tag> {
    view.read_with(cx, |view, _| {
        view.store
            .list_tags()
            .unwrap()
            .into_iter()
            .map(|summary| summary.tag)
            .find(|tag| tag.name == name)
    })
}

fn note_of(cx: &mut TestAppContext, view: &Entity<Workspace>, id: &NoteId) -> Note {
    view.read_with(cx, |view, _| view.store.find_note(id).unwrap().unwrap())
}

#[gpui::test]
fn a_collapsed_notebook_lists_its_sub_notebooks_notes_too(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, &row(&tree.work));

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.work.id, NotebookDepth::WithDescendants)
    );
    assert_eq!(listed_titles(cx, &view), ["In sub", "In work"]);
}

#[gpui::test]
fn an_expanded_notebook_lists_only_its_direct_notes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    toggle_notebook(cx, handle, &tree.work);
    click(cx, handle, &row(&tree.work));

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.work.id, NotebookDepth::DirectOnly)
    );
    assert_eq!(listed_titles(cx, &view), ["In work"]);
}

#[gpui::test]
fn expanding_the_selected_notebook_narrows_the_list_and_collapsing_widens_it(
    cx: &mut TestAppContext,
) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.work));

    toggle_notebook(cx, handle, &tree.work);
    assert_eq!(listed_titles(cx, &view), ["In work"]);

    toggle_notebook(cx, handle, &tree.work);
    assert_eq!(listed_titles(cx, &view), ["In sub", "In work"]);
}

#[gpui::test]
fn expanding_another_notebook_reveals_its_children_and_keeps_the_navigation(
    cx: &mut TestAppContext,
) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    assert!(!is_listed(cx, handle, &row(&tree.sub)));

    toggle_notebook(cx, handle, &tree.work);

    assert!(is_listed(cx, handle, &row(&tree.sub)));
    assert_eq!(navigation(cx, &view), Navigation::AllNotes);

    toggle_notebook(cx, handle, &tree.work);
    assert!(!is_listed(cx, handle, &row(&tree.sub)));
}

#[gpui::test]
fn expanding_a_notebook_keeps_another_selected_notebook_selected(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.other));

    toggle_notebook(cx, handle, &tree.work);

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.other.id, NotebookDepth::WithDescendants)
    );
}

#[gpui::test]
fn the_plus_button_creates_a_top_level_notebook(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    type_name(cx, handle, "Ideas");
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(
        notebook_names(cx, &view),
        ["Ideas", "Inbox", "Other", "Work", "Sub"]
    );
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
    assert!(!banner_is_shown(cx, handle));
}

#[gpui::test]
fn pressing_enter_confirms_the_name(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    type_name(cx, handle, "Ideas");
    press(cx, handle, "enter");

    assert!(notebook_names(cx, &view).contains(&"Ideas".to_owned()));
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn an_empty_name_is_refused_with_the_banner_and_keeps_the_prompt(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(banner_is_shown(cx, handle));
    assert!(is_listed(cx, handle, "sidebar-prompt"));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);

    type_name(cx, handle, "Ideas");
    click(cx, handle, "sidebar-prompt-confirm");
    assert!(!banner_is_shown(cx, handle));
    assert!(notebook_names(cx, &view).contains(&"Ideas".to_owned()));
}

#[gpui::test]
fn escape_in_the_name_field_cancels_the_prompt(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    type_name(cx, handle, "Ideas");
    press(cx, handle, "escape");

    assert!(!is_listed(cx, handle, "sidebar-prompt"));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
}

#[gpui::test]
fn cancelling_the_prompt_changes_nothing(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    type_name(cx, handle, "Ideas");
    click(cx, handle, "sidebar-prompt-cancel");

    assert!(!is_listed(cx, handle, "sidebar-prompt"));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
}

#[gpui::test]
fn the_context_menu_creates_a_sub_notebook_and_reveals_it(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.work));

    choose_menu_item(cx, handle, &row(&tree.work), NEW_SUB_NOTEBOOK);
    type_name(cx, handle, "Deep");
    click(cx, handle, "sidebar-prompt-confirm");

    let work_children = view.read_with(cx, |view, _| {
        let work = view
            .notebooks
            .iter()
            .find(|node| node.notebook.id == tree.work.id)
            .unwrap();
        work.children
            .iter()
            .map(|node| node.notebook.name.clone())
            .collect::<Vec<_>>()
    });
    assert_eq!(work_children, ["Deep", "Sub"]);
    assert!(is_listed(cx, handle, &row(&tree.sub)));
    assert_eq!(listed_titles(cx, &view), ["In work"]);
}

#[gpui::test]
fn the_context_menu_renames_a_notebook_starting_from_its_name(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), RENAME_NOTEBOOK);
    press(cx, handle, "end");
    type_text(cx, handle, "s");
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(
        notebook_names(cx, &view),
        ["Inbox", "Other", "Works", "Sub"]
    );
}

#[gpui::test]
fn renaming_a_notebook_to_nothing_is_refused(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), RENAME_NOTEBOOK);
    press(cx, handle, "ctrl-a");
    press(cx, handle, "backspace");
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(banner_is_shown(cx, handle));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
}

#[gpui::test]
fn deleting_a_notebook_asks_first_and_moves_its_notes_to_trash(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);

    assert!(is_listed(cx, handle, "sidebar-prompt"));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
    let question = view.read_with(cx, |view, _| match &view.sidebar.prompt {
        Some(crate::sidebar::Prompt::Deletion { question, .. }) => Some(question.to_string()),
        _ => None,
    });
    assert_eq!(
        question.as_deref(),
        Some("Delete notebook Work? Its notes move to Trash.")
    );

    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other"]);
    assert_eq!(listed_titles(cx, &view), ["In inbox"]);
    click(cx, handle, "nav-trash");
    assert_eq!(listed_titles(cx, &view), ["In sub", "In work"]);
}

#[gpui::test]
fn deleting_a_notebook_saves_the_pending_edit_and_clears_the_selection(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    let edited = cx.read(|cx| view.read(cx).selected.clone().unwrap());
    super::type_into_title(cx, handle, "!");

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    let note = note_of(cx, &view, &edited);
    assert_eq!(note.title, "In sub!");
    assert!(note.trashed_at.is_some());
    assert_eq!(cx.read(|cx| view.read(cx).selected.clone()), None);
}

#[gpui::test]
fn declining_the_deletion_keeps_the_notebook(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-cancel");

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn deleting_the_selected_notebook_navigates_to_all_notes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.work));

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    assert_eq!(listed_titles(cx, &view), ["In inbox"]);
}

#[gpui::test]
fn deleting_a_parent_of_the_selected_notebook_navigates_to_all_notes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    toggle_notebook(cx, handle, &tree.work);
    click(cx, handle, &row(&tree.sub));

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
}

#[gpui::test]
fn deleting_another_notebook_keeps_the_navigation(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    toggle_notebook(cx, handle, &tree.work);
    click(cx, handle, &row(&tree.sub));

    choose_menu_item(cx, handle, &row(&tree.other), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.sub.id, NotebookDepth::WithDescendants)
    );
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Work", "Sub"]);
}

#[gpui::test]
fn the_inbox_menu_has_no_delete_item(cx: &mut TestAppContext) {
    let tree = tree_store();
    let inbox = tree.store.default_notebook().unwrap();
    let (handle, view) = open_view(cx, tree.store);

    assert_eq!(
        menu_labels_of(cx, handle, &view, &format!("nav-notebook-{inbox}")),
        ["New sub-notebook", "Rename"]
    );
}

#[gpui::test]
fn other_notebook_menus_offer_sub_notebook_rename_default_and_delete(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    assert_eq!(
        menu_labels_of(cx, handle, &view, &row(&tree.other)),
        ["New sub-notebook", "Rename", "Use for new notes", "Delete"]
    );
}

#[gpui::test]
fn the_tag_menu_offers_rename_colour_and_delete(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    assert_eq!(
        menu_labels_of(cx, handle, &view, &tag_row(&tagged.idea)),
        ["Rename", "Change colour", "Delete"]
    );
}

struct Tagged {
    store: NoteStore,
    idea: Tag,
    urgent: Tag,
    both: NoteId,
}

fn tagged_store() -> Tagged {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let idea = store.create_tag("idea", TagColor::Red, at(1)).unwrap();
    let urgent = store.create_tag("urgent", TagColor::Blue, at(1)).unwrap();
    let only_idea = note_in(&store, &inbox, "Only idea", 2);
    let both = note_in(&store, &inbox, "Both", 3);
    let only_urgent = note_in(&store, &inbox, "Only urgent", 4);
    store.add_tag_to_note(&only_idea, &idea.id, at(5)).unwrap();
    store.add_tag_to_note(&both, &idea.id, at(5)).unwrap();
    store.add_tag_to_note(&both, &urgent.id, at(5)).unwrap();
    store
        .add_tag_to_note(&only_urgent, &urgent.id, at(5))
        .unwrap();
    Tagged {
        store,
        idea,
        urgent,
        both,
    }
}

#[gpui::test]
fn clicking_a_tag_selects_it_alone(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    click(cx, handle, &tag_row(&tagged.idea));
    click(cx, handle, &tag_row(&tagged.urgent));

    assert_eq!(
        navigation(cx, &view),
        Navigation::Tags(vec![tagged.urgent.id])
    );
    assert_eq!(listed_titles(cx, &view), ["Only urgent", "Both"]);
}

#[gpui::test]
fn ctrl_clicking_tags_lists_only_notes_with_all_of_them(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    click(cx, handle, &tag_row(&tagged.idea));
    ctrl_click(cx, handle, &tag_row(&tagged.urgent));

    assert_eq!(
        navigation(cx, &view),
        Navigation::Tags(vec![tagged.idea.id.clone(), tagged.urgent.id.clone()])
    );
    assert_eq!(listed_titles(cx, &view), ["Both"]);

    ctrl_click(cx, handle, &tag_row(&tagged.idea));
    assert_eq!(listed_titles(cx, &view), ["Only urgent", "Both"]);
}

#[gpui::test]
fn ctrl_clicking_the_only_selected_tag_returns_to_all_notes(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    click(cx, handle, &tag_row(&tagged.idea));
    ctrl_click(cx, handle, &tag_row(&tagged.idea));

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
}

#[gpui::test]
fn the_tag_menu_renames_a_tag(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), RENAME_TAG);
    type_name(cx, handle, "thought");
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(stored_tag(cx, &view, "thought").is_some());
    assert!(stored_tag(cx, &view, "idea").is_none());
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn renaming_a_tag_to_an_existing_name_is_refused(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), RENAME_TAG);
    type_name(cx, handle, "urgent");
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(banner_is_shown(cx, handle));
    assert!(stored_tag(cx, &view, "idea").is_some());
}

#[gpui::test]
fn the_tag_menu_changes_the_colour_through_a_submenu(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);
    let row = tag_row(&tagged.idea);

    choose_menu_item(cx, handle, &row, CHANGE_COLOUR);
    assert_eq!(menu_labels(cx, &view).len(), TagColor::ALL.len());
    choose_menu_entry(cx, handle, green_position());

    assert_eq!(
        stored_tag(cx, &view, "idea").unwrap().color,
        TagColor::Green
    );
    let shown = view.read_with(cx, |view, _| {
        view.tags
            .iter()
            .find(|summary| summary.tag.name == "idea")
            .unwrap()
            .tag
            .color
    });
    assert_eq!(shown, TagColor::Green);
}

#[gpui::test]
fn deleting_a_tag_asks_first_and_removes_it_from_the_notes(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), DELETE_TAG);
    assert!(stored_tag(cx, &view, "idea").is_some());
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(stored_tag(cx, &view, "idea").is_none());
    assert!(!is_listed(cx, handle, &tag_row(&tagged.idea)));
    assert_eq!(note_of(cx, &view, &tagged.both).tags.len(), 1);
}

#[gpui::test]
fn deleting_the_selected_tag_navigates_to_all_notes(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);
    click(cx, handle, &tag_row(&tagged.idea));

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), DELETE_TAG);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    assert_eq!(listed_titles(cx, &view).len(), 3);
}

#[gpui::test]
fn deleting_one_of_two_selected_tags_keeps_the_other(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);
    click(cx, handle, &tag_row(&tagged.idea));
    ctrl_click(cx, handle, &tag_row(&tagged.urgent));

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), DELETE_TAG);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(
        navigation(cx, &view),
        Navigation::Tags(vec![tagged.urgent.id])
    );
    assert_eq!(listed_titles(cx, &view), ["Only urgent", "Both"]);
}

#[gpui::test]
fn status_rows_and_trash_navigate(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    click(cx, handle, "nav-status-on_hold");
    assert_eq!(
        navigation(cx, &view),
        Navigation::Status(NoteStatus::OnHold)
    );
    click(cx, handle, "nav-trash");
    assert_eq!(navigation(cx, &view), Navigation::Trash);
    click(cx, handle, "nav-all-notes");
    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
}

#[gpui::test]
fn ctrl_slash_hides_and_shows_the_sidebar(cx: &mut TestAppContext) {
    let (handle, _) = open_view(cx, NoteStore::open_in_memory().unwrap());
    assert!(is_listed(cx, handle, "sidebar"));

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-/");
    assert!(!is_listed(cx, handle, "sidebar"));
    assert!(is_listed(cx, handle, "new-note"));

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-/");
    assert!(is_listed(cx, handle, "sidebar"));
}

#[gpui::test]
fn ctrl_slash_works_while_the_title_has_focus(cx: &mut TestAppContext) {
    let (store, _) = super::seeded_store();
    let (handle, _) = open_view(cx, store);
    click(cx, handle, "title");

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-/");

    assert!(!is_listed(cx, handle, "sidebar"));
}

#[gpui::test]
fn making_a_notebook_the_default_sends_new_notes_there(cx: &mut TestAppContext) {
    let (store, _) = super::seeded_store();
    let work = store.create_notebook("Work", None, at(7)).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-all-notes");

    view.update(cx, |view, cx| view.make_default_notebook(&work.id, cx));
    click(cx, handle, "new-note");

    let notebook = view.read_with(cx, |view, _| {
        let created = view.selected.clone().unwrap();
        view.store.find_note(&created).unwrap().unwrap().notebook
    });
    assert_eq!(notebook, work.id);
}

#[gpui::test]
fn collapsing_a_section_hides_its_rows_and_expanding_brings_them_back(cx: &mut TestAppContext) {
    let (handle, _) = open_view(cx, NoteStore::open_in_memory().unwrap());
    assert!(is_listed(cx, handle, "nav-status-active"));

    click_section_collapse(cx, handle, "status-header");
    assert!(!is_listed(cx, handle, "nav-status-active"));
    assert!(is_listed(cx, handle, "nav-trash"));

    click_section_collapse(cx, handle, "status-header");
    assert!(is_listed(cx, handle, "nav-status-active"));
}

#[gpui::test]
fn the_note_list_toggle_button_hides_and_shows_the_sidebar(cx: &mut TestAppContext) {
    let (handle, _) = open_view(cx, NoteStore::open_in_memory().unwrap());

    click(cx, handle, "toggle-sidebar");
    assert!(!is_listed(cx, handle, "sidebar"));

    click(cx, handle, "toggle-sidebar");
    assert!(is_listed(cx, handle, "sidebar"));
}

#[gpui::test]
fn a_context_menu_is_our_menu_and_closes_once_an_entry_ran(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.other));

    open_menu(cx, handle, &row(&tree.work));
    assert!(view.read_with(cx, |view, _| view.open_menu.is_some()));
    assert_eq!(menu_labels(cx, &view).len(), 4);

    choose_menu_entry(cx, handle, RENAME_NOTEBOOK);

    assert!(view.read_with(cx, |view, _| view.open_menu.is_none()));
    assert!(is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn escape_closes_a_context_menu_without_running_anything(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    open_menu(cx, handle, &row(&tree.work));
    press(cx, handle, "escape");
    cx.run_until_parked();

    assert!(view.read_with(cx, |view, _| view.open_menu.is_none()));
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn the_menu_hands_the_focus_back_to_the_sidebar_when_it_closes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.other));

    choose_menu_item(cx, handle, &row(&tree.work), 2);

    let focused = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            view.read(cx).sidebar_focus.contains_focused(window, cx)
        })
        .unwrap();
    assert!(focused);
}

#[gpui::test]
fn enter_confirms_the_deletion_dialog(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("enter");

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other"]);
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn escape_declines_the_deletion_dialog(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("escape");

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn clicking_beside_the_deletion_dialog_declines_it(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click_at(cx, handle, "sidebar-prompt", point(px(5.), px(5.)));

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui::test]
fn shortcuts_work_again_after_a_dialog_closes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("escape");
    render(cx, handle);
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-,");

    assert!(view.read_with(cx, |view, _| view.settings_open));
}

#[gpui::test]
fn shortcuts_work_again_after_a_confirmation_closes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("escape");
    render(cx, handle);
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-,");

    assert!(view.read_with(cx, |view, _| view.settings_open));
}
