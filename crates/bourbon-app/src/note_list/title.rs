use bourbon_core::{NotebookId, NotebookNode, TagId, TagSummary};

use crate::navigation::Navigation;
use crate::presentation::status_label;

pub(crate) fn view_title(
    navigation: &Navigation,
    notebooks: &[NotebookNode],
    tags: &[TagSummary],
) -> String {
    match navigation {
        Navigation::AllNotes => "All Notes".to_owned(),
        Navigation::Notebook(id, _) => notebook_name(id, notebooks).unwrap_or_default(),
        Navigation::Status(status) => status_label(*status).to_owned(),
        Navigation::Tags(ids) => tag_names(ids, tags),
        Navigation::Trash => "Trash".to_owned(),
    }
}

fn notebook_name(id: &NotebookId, nodes: &[NotebookNode]) -> Option<String> {
    nodes.iter().find_map(|node| {
        if node.notebook.id == *id {
            Some(node.notebook.name.clone())
        } else {
            notebook_name(id, &node.children)
        }
    })
}

fn tag_names(ids: &[TagId], tags: &[TagSummary]) -> String {
    ids.iter()
        .filter_map(|id| tags.iter().find(|summary| summary.tag.id == *id))
        .map(|summary| summary.tag.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use bourbon_core::{NoteStatus, Notebook, NotebookDepth, Tag, TagColor, Timestamp};

    use super::*;

    fn node(name: &str, children: Vec<NotebookNode>) -> NotebookNode {
        NotebookNode {
            notebook: Notebook {
                id: NotebookId::generate(),
                name: name.to_owned(),
                parent: None,
                created_at: Timestamp::from_millis(0),
            },
            note_count: 0,
            children,
        }
    }

    fn summary(name: &str) -> TagSummary {
        TagSummary {
            tag: Tag {
                id: TagId::generate(),
                name: name.to_owned(),
                color: TagColor::Default,
            },
            note_count: 0,
        }
    }

    #[test]
    fn fixed_views_carry_their_names() {
        assert_eq!(view_title(&Navigation::AllNotes, &[], &[]), "All Notes");
        assert_eq!(view_title(&Navigation::Trash, &[], &[]), "Trash");
        assert_eq!(
            view_title(&Navigation::Status(NoteStatus::OnHold), &[], &[]),
            "On Hold"
        );
    }

    #[test]
    fn a_nested_notebook_is_titled_with_its_own_name() {
        let child = node("Recipes", Vec::new());
        let child_id = child.notebook.id.clone();
        let tree = [node("Home", vec![child])];
        assert_eq!(
            view_title(
                &Navigation::Notebook(child_id, NotebookDepth::WithDescendants),
                &tree,
                &[]
            ),
            "Recipes"
        );
    }

    #[test]
    fn an_unknown_notebook_has_an_empty_title() {
        assert_eq!(
            view_title(
                &Navigation::Notebook(NotebookId::generate(), NotebookDepth::WithDescendants),
                &[],
                &[]
            ),
            ""
        );
    }

    #[test]
    fn selected_tags_are_listed_in_selection_order() {
        let tags = [summary("work"), summary("idea"), summary("later")];
        let selection = vec![tags[2].tag.id.clone(), tags[0].tag.id.clone()];
        assert_eq!(
            view_title(&Navigation::Tags(selection), &[], &tags),
            "later, work"
        );
    }
}
