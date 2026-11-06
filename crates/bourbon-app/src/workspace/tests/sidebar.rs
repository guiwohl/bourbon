use bourbon_core::{Note, NoteId, NoteStatus, NoteStore, Notebook, NotebookDepth, Tag, TagColor};
use gpui_kit::base::test_support::texts;
use gpui_kit::test::{ClickOptions, TestWindowExt as _};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext,
};

use super::{at, banner_is_shown, click, listed_titles, note_in, open_view};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

const NEW_SUB_NOTEBOOK: usize = 0;
const RENAME_NOTEBOOK: usize = 1;
const DELETE_NOTEBOOK: usize = 2;
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

fn toggle(notebook: &Notebook) -> String {
    format!("toggle-notebook-{}", notebook.id)
}

fn tag_row(tag: &Tag) -> String {
    format!("nav-tag-{}", tag.id)
}

fn ctrl_click(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let options = ClickOptions::new().with_modifiers(Modifiers::control());
        window.click_with_options(id.to_owned(), options, cx);
    })
    .unwrap();
}

fn open_menu(cx: &mut TestAppContext, handle: AnyWindowHandle, row: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.right_click(row.to_owned(), cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn choose_menu_item(cx: &mut TestAppContext, handle: AnyWindowHandle, row: &str, index: usize) {
    open_menu(cx, handle, row);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.within("popup-menu").hover(index, cx);
        window.press("enter", cx);
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn menu_labels(cx: &mut TestAppContext, handle: AnyWindowHandle, row: &str) -> Vec<String> {
    open_menu(cx, handle, row);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let menu = window.within("popup-menu");
        (0usize..)
            .map_while(|index| menu.try_find(index))
            .map(|item| item.label().unwrap_or_default().to_string())
            .collect()
    })
    .unwrap()
}

fn type_name(cx: &mut TestAppContext, handle: AnyWindowHandle, name: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-a", cx);
        window.input(name, cx);
    })
    .unwrap();
}

fn is_listed(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id.to_owned()).is_some()
    })
    .unwrap()
}

fn texts_in_row(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> Vec<String> {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let bounds = window.find(id.to_owned()).bounds();
        texts(window)
            .iter()
            .filter(|snapshot| bounds.contains(&snapshot.bounds().center()))
            .map(|snapshot| snapshot.text().to_string())
            .collect()
    })
    .unwrap()
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

#[gpui_kit::test]
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

#[gpui_kit::test]
fn an_expanded_notebook_lists_only_its_direct_notes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, &toggle(&tree.work));
    click(cx, handle, &row(&tree.work));

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.work.id, NotebookDepth::DirectOnly)
    );
    assert_eq!(listed_titles(cx, &view), ["In work"]);
}

#[gpui_kit::test]
fn expanding_the_selected_notebook_narrows_the_list_and_collapsing_widens_it(
    cx: &mut TestAppContext,
) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.work));

    click(cx, handle, &toggle(&tree.work));
    assert_eq!(listed_titles(cx, &view), ["In work"]);

    click(cx, handle, &toggle(&tree.work));
    assert_eq!(listed_titles(cx, &view), ["In sub", "In work"]);
}

#[gpui_kit::test]
fn expanding_another_notebook_reveals_its_children_and_keeps_the_navigation(
    cx: &mut TestAppContext,
) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    assert!(!is_listed(cx, handle, &row(&tree.sub)));

    click(cx, handle, &toggle(&tree.work));

    assert!(is_listed(cx, handle, &row(&tree.sub)));
    assert_eq!(navigation(cx, &view), Navigation::AllNotes);

    click(cx, handle, &toggle(&tree.work));
    assert!(!is_listed(cx, handle, &row(&tree.sub)));
}

#[gpui_kit::test]
fn expanding_a_notebook_keeps_another_selected_notebook_selected(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.other));

    click(cx, handle, &toggle(&tree.work));

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.other.id, NotebookDepth::WithDescendants)
    );
}

#[gpui_kit::test]
fn notebooks_without_children_have_no_disclosure(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, _) = open_view(cx, tree.store);

    assert!(is_listed(cx, handle, &toggle(&tree.work)));
    click(cx, handle, &toggle(&tree.work));
    assert!(!is_listed(cx, handle, &toggle(&tree.sub)));
}

#[gpui_kit::test]
fn every_notebook_row_shows_its_name_and_note_count(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, _) = open_view(cx, tree.store);
    click(cx, handle, &toggle(&tree.work));

    assert_eq!(texts_in_row(cx, handle, &row(&tree.work)), ["2"]);
    assert_eq!(texts_in_row(cx, handle, &row(&tree.sub)), ["1"]);
}

#[gpui_kit::test]
fn every_tag_row_shows_its_note_count(cx: &mut TestAppContext) {
    let tree = tree_store();
    let idea = tree.store.create_tag("idea", TagColor::Red, at(6)).unwrap();
    for title in ["In inbox", "In work"] {
        let note = tree
            .store
            .list_notes(&bourbon_core::NoteQuery::default())
            .unwrap()
            .into_iter()
            .find(|summary| summary.title == title)
            .unwrap();
        tree.store
            .add_tag_to_note(&note.id, &idea.id, at(7))
            .unwrap();
    }
    let (handle, _) = open_view(cx, tree.store);

    assert_eq!(texts_in_row(cx, handle, &tag_row(&idea)), ["2"]);
}

#[gpui_kit::test]
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

#[gpui_kit::test]
fn pressing_enter_confirms_the_name(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    type_name(cx, handle, "Ideas");
    cx.update_window(handle, |_, window, cx| window.press("enter", cx))
        .unwrap();

    assert!(notebook_names(cx, &view).contains(&"Ideas".to_owned()));
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui_kit::test]
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

#[gpui_kit::test]
fn cancelling_the_prompt_changes_nothing(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    click(cx, handle, "new-notebook");
    type_name(cx, handle, "Ideas");
    click(cx, handle, "sidebar-prompt-cancel");

    assert!(!is_listed(cx, handle, "sidebar-prompt"));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
}

#[gpui_kit::test]
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

#[gpui_kit::test]
fn the_context_menu_renames_a_notebook_starting_from_its_name(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), RENAME_NOTEBOOK);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.press("end", cx);
        window.input("s", cx);
    })
    .unwrap();
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(
        notebook_names(cx, &view),
        ["Inbox", "Other", "Works", "Sub"]
    );
}

#[gpui_kit::test]
fn renaming_a_notebook_to_nothing_is_refused(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), RENAME_NOTEBOOK);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-a", cx);
        window.press("backspace", cx);
    })
    .unwrap();
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(banner_is_shown(cx, handle));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
}

#[gpui_kit::test]
fn deleting_a_notebook_asks_first_and_moves_its_notes_to_trash(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);

    assert!(is_listed(cx, handle, "sidebar-prompt"));
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
    let prompt_text = cx
        .update_window(handle, |_, window, _| {
            texts(window)
                .iter()
                .map(|snapshot| snapshot.text().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert!(prompt_text.contains(&"Delete notebook Work? Its notes move to Trash.".to_owned()));

    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other"]);
    assert_eq!(listed_titles(cx, &view), ["In inbox"]);
    click(cx, handle, "nav-trash");
    assert_eq!(listed_titles(cx, &view), ["In sub", "In work"]);
}

#[gpui_kit::test]
fn declining_the_deletion_keeps_the_notebook(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-cancel");

    assert_eq!(notebook_names(cx, &view), ["Inbox", "Other", "Work", "Sub"]);
    assert!(!is_listed(cx, handle, "sidebar-prompt"));
}

#[gpui_kit::test]
fn deleting_the_selected_notebook_navigates_to_all_notes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &row(&tree.work));

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    assert_eq!(listed_titles(cx, &view), ["In inbox"]);
}

#[gpui_kit::test]
fn deleting_a_parent_of_the_selected_notebook_navigates_to_all_notes(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &toggle(&tree.work));
    click(cx, handle, &row(&tree.sub));

    choose_menu_item(cx, handle, &row(&tree.work), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
}

#[gpui_kit::test]
fn deleting_another_notebook_keeps_the_navigation(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, view) = open_view(cx, tree.store);
    click(cx, handle, &toggle(&tree.work));
    click(cx, handle, &row(&tree.sub));

    choose_menu_item(cx, handle, &row(&tree.other), DELETE_NOTEBOOK);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(
        navigation(cx, &view),
        Navigation::Notebook(tree.sub.id, NotebookDepth::WithDescendants)
    );
    assert_eq!(notebook_names(cx, &view), ["Inbox", "Work", "Sub"]);
}

#[gpui_kit::test]
fn the_inbox_menu_has_no_delete_item(cx: &mut TestAppContext) {
    let tree = tree_store();
    let inbox = tree.store.default_notebook().unwrap();
    let (handle, _) = open_view(cx, tree.store);

    assert_eq!(
        menu_labels(cx, handle, &format!("nav-notebook-{inbox}")),
        ["New sub-notebook", "Rename"]
    );
}

#[gpui_kit::test]
fn other_notebook_menus_offer_new_sub_notebook_rename_and_delete(cx: &mut TestAppContext) {
    let tree = tree_store();
    let (handle, _) = open_view(cx, tree.store);

    assert_eq!(
        menu_labels(cx, handle, &row(&tree.other)),
        ["New sub-notebook", "Rename", "Delete"]
    );
}

#[gpui_kit::test]
fn the_tag_menu_offers_rename_colour_and_delete(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, _) = open_view(cx, tagged.store);

    assert_eq!(
        menu_labels(cx, handle, &tag_row(&tagged.idea)),
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

#[gpui_kit::test]
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

#[gpui_kit::test]
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

#[gpui_kit::test]
fn ctrl_clicking_the_only_selected_tag_returns_to_all_notes(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    click(cx, handle, &tag_row(&tagged.idea));
    ctrl_click(cx, handle, &tag_row(&tagged.idea));

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
}

#[gpui_kit::test]
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

#[gpui_kit::test]
fn renaming_a_tag_to_an_existing_name_is_refused(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), RENAME_TAG);
    type_name(cx, handle, "urgent");
    click(cx, handle, "sidebar-prompt-confirm");

    assert!(banner_is_shown(cx, handle));
    assert!(stored_tag(cx, &view, "idea").is_some());
}

#[gpui_kit::test]
fn the_tag_menu_changes_the_colour_through_a_submenu(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);
    let row = tag_row(&tagged.idea);

    open_menu(cx, handle, &row);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.within("popup-menu").hover(CHANGE_COLOUR, cx);
        window.press("enter", cx);
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.within("submenu").hover(green_position(), cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();

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

#[gpui_kit::test]
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

#[gpui_kit::test]
fn deleting_the_selected_tag_navigates_to_all_notes(cx: &mut TestAppContext) {
    let tagged = tagged_store();
    let (handle, view) = open_view(cx, tagged.store);
    click(cx, handle, &tag_row(&tagged.idea));

    choose_menu_item(cx, handle, &tag_row(&tagged.idea), DELETE_TAG);
    click(cx, handle, "sidebar-prompt-confirm");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    assert_eq!(listed_titles(cx, &view).len(), 3);
}

#[gpui_kit::test]
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

#[gpui_kit::test]
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

#[gpui_kit::test]
fn ctrl_slash_hides_and_shows_the_sidebar(cx: &mut TestAppContext) {
    let (handle, _) = open_view(cx, NoteStore::open_in_memory().unwrap());
    assert!(is_listed(cx, handle, "sidebar"));

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-/");
    assert!(!is_listed(cx, handle, "sidebar"));
    assert!(is_listed(cx, handle, "new-note"));

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-/");
    assert!(is_listed(cx, handle, "sidebar"));
}

#[gpui_kit::test]
fn ctrl_slash_works_while_the_title_has_focus(cx: &mut TestAppContext) {
    let (store, _) = super::seeded_store();
    let (handle, _) = open_view(cx, store);
    click(cx, handle, "title");

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-/");

    assert!(!is_listed(cx, handle, "sidebar"));
}
