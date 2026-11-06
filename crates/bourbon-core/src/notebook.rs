use std::collections::HashMap;

use crate::identifier::NotebookId;
use crate::timestamp::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notebook {
    pub id: NotebookId,
    pub name: String,
    pub parent: Option<NotebookId>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotebookNode {
    pub notebook: Notebook,
    pub note_count: usize,
    pub children: Vec<NotebookNode>,
}

pub(crate) fn build_tree(
    notebooks: Vec<Notebook>,
    direct_note_counts: &HashMap<NotebookId, usize>,
) -> Vec<NotebookNode> {
    let mut by_parent: HashMap<Option<NotebookId>, Vec<Notebook>> = HashMap::new();
    for notebook in notebooks {
        by_parent
            .entry(notebook.parent.clone())
            .or_default()
            .push(notebook);
    }
    for siblings in by_parent.values_mut() {
        siblings.sort_by_cached_key(|notebook| {
            (
                notebook.name.to_lowercase(),
                notebook.created_at,
                notebook.id.to_string(),
            )
        });
    }
    let roots = by_parent.remove(&None).unwrap_or_default();
    nodes_of(roots, &mut by_parent, direct_note_counts)
}

fn nodes_of(
    siblings: Vec<Notebook>,
    by_parent: &mut HashMap<Option<NotebookId>, Vec<Notebook>>,
    direct_note_counts: &HashMap<NotebookId, usize>,
) -> Vec<NotebookNode> {
    siblings
        .into_iter()
        .map(|notebook| {
            let own_children = by_parent
                .remove(&Some(notebook.id.clone()))
                .unwrap_or_default();
            let children = nodes_of(own_children, by_parent, direct_note_counts);
            let own_count = direct_note_counts.get(&notebook.id).copied().unwrap_or(0);
            let note_count =
                own_count + children.iter().map(|child| child.note_count).sum::<usize>();
            NotebookNode {
                notebook,
                note_count,
                children,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notebook(name: &str, parent: Option<&NotebookId>, created: i64) -> Notebook {
        Notebook {
            id: NotebookId::generate(),
            name: name.to_owned(),
            parent: parent.cloned(),
            created_at: Timestamp::from_millis(created),
        }
    }

    fn names(nodes: &[NotebookNode]) -> Vec<&str> {
        nodes
            .iter()
            .map(|node| node.notebook.name.as_str())
            .collect()
    }

    #[test]
    fn roots_and_children_are_ordered_by_name_ignoring_case() {
        let work = notebook("work", None, 1);
        let archive = notebook("Archive", None, 2);
        let zeta = notebook("zeta", Some(&work.id), 3);
        let alpha = notebook("Alpha", Some(&work.id), 4);
        let tree = build_tree(vec![zeta, work, alpha, archive], &HashMap::new());
        assert_eq!(names(&tree), ["Archive", "work"]);
        assert_eq!(names(&tree[1].children), ["Alpha", "zeta"]);
    }

    #[test]
    fn equal_names_are_ordered_by_creation() {
        let later = notebook("Same", None, 9);
        let earlier = notebook("same", None, 2);
        let tree = build_tree(vec![later.clone(), earlier.clone()], &HashMap::new());
        let ids: Vec<_> = tree.iter().map(|node| node.notebook.id.clone()).collect();
        assert_eq!(ids, [earlier.id, later.id]);
    }

    #[test]
    fn note_count_includes_all_descendants() {
        let root = notebook("Root", None, 1);
        let child = notebook("Child", Some(&root.id), 2);
        let grandchild = notebook("Grandchild", Some(&child.id), 3);
        let counts = HashMap::from([
            (root.id.clone(), 1),
            (child.id.clone(), 2),
            (grandchild.id.clone(), 4),
        ]);
        let tree = build_tree(vec![grandchild, root, child], &counts);
        assert_eq!(tree[0].note_count, 7);
        assert_eq!(tree[0].children[0].note_count, 6);
        assert_eq!(tree[0].children[0].children[0].note_count, 4);
        assert_eq!(tree[0].children[0].children[0].children.len(), 0);
    }

    #[test]
    fn notebooks_without_notes_count_zero() {
        let tree = build_tree(vec![notebook("Empty", None, 1)], &HashMap::new());
        assert_eq!(tree[0].note_count, 0);
    }

    #[test]
    fn no_notebooks_give_an_empty_tree() {
        assert_eq!(build_tree(Vec::new(), &HashMap::new()).len(), 0);
    }
}
