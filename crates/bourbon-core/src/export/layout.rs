use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::document;
use super::slug::{folder_name_of, slug_of};
use crate::identifier::NotebookId;
use crate::note::Note;
use crate::notebook::Notebook;

const TEMPLATES_FOLDER: &str = "_templates";
const NOTE_EXTENSION: &str = ".md";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExportSummary {
    pub notebooks: usize,
    pub notes: usize,
    pub templates: usize,
}

#[derive(Debug, Default)]
pub struct ExportPlan {
    pub directories: Vec<PathBuf>,
    pub files: Vec<ExportFile>,
    pub summary: ExportSummary,
}

#[derive(Debug)]
pub struct ExportFile {
    pub path: PathBuf,
    pub contents: String,
}

pub fn plan(notebooks: &[Notebook], notes: &[Note]) -> ExportPlan {
    let mut plan = ExportPlan::default();
    let folders = notebook_folders(notebooks);
    plan.summary.notebooks = folders.len();
    plan.directories.extend(folders.values().cloned());

    let mut notes_by_folder: BTreeMap<PathBuf, Vec<&Note>> = BTreeMap::new();
    for note in notes {
        let folder = if note.is_template {
            PathBuf::from(TEMPLATES_FOLDER)
        } else {
            folders.get(&note.notebook).cloned().unwrap_or_default()
        };
        notes_by_folder.entry(folder).or_default().push(note);
        if note.is_template {
            plan.summary.templates += 1;
        } else {
            plan.summary.notes += 1;
        }
    }
    if plan.summary.templates > 0 {
        plan.directories.push(PathBuf::from(TEMPLATES_FOLDER));
    }
    for (folder, mut folder_notes) in notes_by_folder {
        folder_notes.sort_by(|left, right| oldest_first(left, right));
        let mut names = UniqueNames::default();
        for note in folder_notes {
            let file_name = names.claim(&slug_of(&note.title), NOTE_EXTENSION);
            plan.files.push(ExportFile {
                path: folder.join(file_name),
                contents: document::render(note),
            });
        }
    }
    plan
}

fn oldest_first(left: &Note, right: &Note) -> std::cmp::Ordering {
    (left.created_at, left.id.as_str()).cmp(&(right.created_at, right.id.as_str()))
}

fn notebook_folders(notebooks: &[Notebook]) -> HashMap<NotebookId, PathBuf> {
    let mut by_parent: HashMap<Option<&NotebookId>, Vec<&Notebook>> = HashMap::new();
    for notebook in notebooks {
        by_parent
            .entry(notebook.parent.as_ref())
            .or_default()
            .push(notebook);
    }
    let mut folders = HashMap::new();
    let mut names = UniqueNames::default();
    names.claim(TEMPLATES_FOLDER, "");
    place_children(None, Path::new(""), &mut names, &by_parent, &mut folders);
    folders
}

fn place_children(
    parent: Option<&NotebookId>,
    parent_folder: &Path,
    names: &mut UniqueNames,
    by_parent: &HashMap<Option<&NotebookId>, Vec<&Notebook>>,
    folders: &mut HashMap<NotebookId, PathBuf>,
) {
    let mut siblings = by_parent.get(&parent).cloned().unwrap_or_default();
    siblings.sort_by_key(|notebook| (notebook.created_at, notebook.id.as_str()));
    for notebook in siblings {
        let folder = parent_folder.join(names.claim(&folder_name_of(&notebook.name), ""));
        folders.insert(notebook.id.clone(), folder.clone());
        place_children(
            Some(&notebook.id),
            &folder,
            &mut UniqueNames::default(),
            by_parent,
            folders,
        );
    }
}

#[derive(Default)]
struct UniqueNames {
    taken: HashSet<String>,
}

impl UniqueNames {
    fn claim(&mut self, base: &str, extension: &str) -> String {
        let name = (0..=self.taken.len())
            .map(|number| match number {
                0 => format!("{base}{extension}"),
                _ => format!("{base}-{number}{extension}"),
            })
            .find(|candidate| !self.taken.contains(candidate))
            .unwrap_or_default();
        self.taken.insert(name.clone());
        name
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{NoteId, NoteStatus, NotebookId, Timestamp};

    fn notebook(name: &str, parent: Option<&Notebook>, created: i64) -> Notebook {
        Notebook {
            id: NotebookId::generate(),
            name: name.to_owned(),
            parent: parent.map(|parent| parent.id.clone()),
            created_at: Timestamp::from_millis(created),
        }
    }

    fn note_in(notebook: &Notebook, title: &str, created: i64) -> Note {
        Note {
            id: NoteId::generate(),
            title: title.to_owned(),
            body: format!("body of {title}"),
            created_at: Timestamp::from_millis(created),
            updated_at: Timestamp::from_millis(created),
            notebook: notebook.id.clone(),
            status: NoteStatus::None,
            pinned: false,
            is_template: false,
            tags: Vec::new(),
            trashed_at: None,
        }
    }

    fn template_in(notebook: &Notebook, title: &str, created: i64) -> Note {
        Note {
            is_template: true,
            ..note_in(notebook, title, created)
        }
    }

    fn file_paths(plan: &ExportPlan) -> Vec<String> {
        let mut paths: Vec<String> = plan
            .files
            .iter()
            .map(|file| file.path.display().to_string())
            .collect();
        paths.sort();
        paths
    }

    fn directories(plan: &ExportPlan) -> Vec<String> {
        let mut paths: Vec<String> = plan
            .directories
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        paths.sort();
        paths
    }

    #[test]
    fn nested_notebooks_become_nested_folders_holding_their_notes() {
        let work = notebook("Work", None, 1);
        let projects = notebook("Projects", Some(&work), 2);
        let notes = [
            note_in(&work, "Standup", 3),
            note_in(&projects, "Bourbon Plan", 4),
        ];
        let plan = plan(&[work, projects], &notes);
        assert_eq!(directories(&plan), ["Work", "Work/Projects"]);
        assert_eq!(
            file_paths(&plan),
            ["Work/Projects/bourbon-plan.md", "Work/standup.md"]
        );
        assert_eq!(
            plan.summary,
            ExportSummary {
                notebooks: 2,
                notes: 2,
                templates: 0
            }
        );
    }

    #[test]
    fn file_contents_are_the_rendered_documents() {
        let inbox = notebook("Inbox", None, 1);
        let only = note_in(&inbox, "Only", 2);
        let plan = plan(&[inbox], std::slice::from_ref(&only));
        assert_eq!(
            plan.files[0].contents,
            super::super::document::render(&only)
        );
    }

    #[test]
    fn empty_notebooks_still_get_a_folder() {
        let empty = notebook("Empty", None, 1);
        let plan = plan(&[empty], &[]);
        assert_eq!(directories(&plan), ["Empty"]);
        assert!(plan.files.is_empty());
    }

    #[test]
    fn duplicate_titles_are_numbered_by_age_then_id() {
        let inbox = notebook("Inbox", None, 1);
        let youngest = note_in(&inbox, "Same Title", 30);
        let oldest = note_in(&inbox, "same title!", 10);
        let middle = note_in(&inbox, "Same-Title", 20);
        let plan = plan(
            &[inbox],
            &[youngest.clone(), oldest.clone(), middle.clone()],
        );
        let path_of = |note: &Note| {
            plan.files
                .iter()
                .find(|file| file.contents.contains(note.id.as_str()))
                .unwrap()
                .path
                .display()
                .to_string()
        };
        assert_eq!(path_of(&oldest), "Inbox/same-title.md");
        assert_eq!(path_of(&middle), "Inbox/same-title-1.md");
        assert_eq!(path_of(&youngest), "Inbox/same-title-2.md");
    }

    #[test]
    fn notes_created_together_are_ordered_by_id() {
        let inbox = notebook("Inbox", None, 1);
        let first = note_in(&inbox, "Tie", 5);
        let second = note_in(&inbox, "Tie", 5);
        let (lower, higher) = if first.id.as_str() < second.id.as_str() {
            (first, second)
        } else {
            (second, first)
        };
        let plan = plan(&[inbox], &[higher.clone(), lower.clone()]);
        let path_of = |note: &Note| {
            plan.files
                .iter()
                .find(|file| file.contents.contains(note.id.as_str()))
                .unwrap()
                .path
                .clone()
        };
        assert_eq!(path_of(&lower), Path::new("Inbox/tie.md"));
        assert_eq!(path_of(&higher), Path::new("Inbox/tie-1.md"));
    }

    #[test]
    fn a_title_that_already_looks_numbered_cannot_collide() {
        let inbox = notebook("Inbox", None, 1);
        let notes = [
            note_in(&inbox, "a", 1),
            note_in(&inbox, "a", 2),
            note_in(&inbox, "a 1", 3),
        ];
        let plan = plan(&[inbox], &notes);
        assert_eq!(
            file_paths(&plan),
            ["Inbox/a-1-1.md", "Inbox/a-1.md", "Inbox/a.md"]
        );
    }

    #[test]
    fn sibling_notebooks_with_the_same_name_are_numbered_by_age() {
        let newer = notebook("Ideas", None, 20);
        let older = notebook("Ideas", None, 10);
        let notes = [note_in(&newer, "N", 1), note_in(&older, "O", 1)];
        let plan = plan(&[newer, older], &notes);
        assert_eq!(directories(&plan), ["Ideas", "Ideas-1"]);
        assert_eq!(file_paths(&plan), ["Ideas-1/n.md", "Ideas/o.md"]);
    }

    #[test]
    fn same_named_notebooks_under_different_parents_do_not_clash() {
        let a = notebook("A", None, 1);
        let b = notebook("B", None, 2);
        let a_inner = notebook("Inner", Some(&a), 3);
        let b_inner = notebook("Inner", Some(&b), 4);
        let plan = plan(&[a, b, a_inner, b_inner], &[]);
        assert_eq!(directories(&plan), ["A", "A/Inner", "B", "B/Inner"]);
    }

    #[test]
    fn notebook_names_are_made_safe_as_folder_names() {
        let odd = notebook("a/b", None, 1);
        let blank = notebook("..", None, 2);
        let plan = plan(&[odd, blank], &[]);
        assert_eq!(directories(&plan), ["a-b", "untitled"]);
    }

    #[test]
    fn templates_go_flat_under_a_templates_folder() {
        let inbox = notebook("Inbox", None, 1);
        let nested = notebook("Deep", Some(&inbox), 2);
        let notes = [
            template_in(&nested, "Meeting", 3),
            template_in(&inbox, "Meeting", 4),
            note_in(&inbox, "Plain", 5),
        ];
        let plan = plan(&[inbox, nested], &notes);
        assert_eq!(
            file_paths(&plan),
            [
                "Inbox/plain.md",
                "_templates/meeting-1.md",
                "_templates/meeting.md"
            ]
        );
        assert_eq!(directories(&plan), ["Inbox", "Inbox/Deep", "_templates"]);
        assert_eq!(
            plan.summary,
            ExportSummary {
                notebooks: 2,
                notes: 1,
                templates: 2
            }
        );
    }

    #[test]
    fn no_templates_means_no_templates_folder() {
        let inbox = notebook("Inbox", None, 1);
        let plan = plan(&[inbox], &[]);
        assert_eq!(directories(&plan), ["Inbox"]);
    }

    #[test]
    fn a_notebook_named_like_the_templates_folder_is_renumbered() {
        let clash = notebook("_templates", None, 1);
        let notes = [template_in(&clash, "T", 2), note_in(&clash, "N", 3)];
        let plan = plan(&[clash], &notes);
        assert_eq!(file_paths(&plan), ["_templates-1/n.md", "_templates/t.md"]);
    }

    #[test]
    fn notes_of_an_unknown_notebook_land_at_the_root() {
        let ghost = notebook("Ghost", None, 1);
        let plan = plan(&[], &[note_in(&ghost, "Lost", 2)]);
        assert_eq!(file_paths(&plan), ["lost.md"]);
    }
}
