use std::collections::HashSet;

use bourbon_core::{NotebookDepth, NotebookId, NotebookNode};

pub(super) struct VisibleNotebook<'a> {
    pub(super) depth: u16,
    pub(super) node: &'a NotebookNode,
    pub(super) is_expanded: bool,
}

pub(super) fn visible_notebooks<'a>(
    nodes: &'a [NotebookNode],
    expanded: &HashSet<NotebookId>,
) -> Vec<VisibleNotebook<'a>> {
    let mut rows = Vec::new();
    collect(nodes, 0, expanded, &mut rows);
    rows
}

fn collect<'a>(
    nodes: &'a [NotebookNode],
    depth: u16,
    expanded: &HashSet<NotebookId>,
    rows: &mut Vec<VisibleNotebook<'a>>,
) {
    for node in nodes {
        let is_expanded = expanded.contains(&node.notebook.id);
        rows.push(VisibleNotebook {
            depth,
            node,
            is_expanded,
        });
        if is_expanded {
            collect(&node.children, depth + 1, expanded, rows);
        }
    }
}

pub(crate) fn depth_for(is_expanded: bool) -> NotebookDepth {
    if is_expanded {
        NotebookDepth::DirectOnly
    } else {
        NotebookDepth::WithDescendants
    }
}

pub(crate) fn subtree_ids(nodes: &[NotebookNode], root: &NotebookId) -> Vec<NotebookId> {
    nodes
        .iter()
        .find_map(|node| {
            if node.notebook.id == *root {
                Some(all_ids(node))
            } else {
                let inside = subtree_ids(&node.children, root);
                (!inside.is_empty()).then_some(inside)
            }
        })
        .unwrap_or_default()
}

fn all_ids(node: &NotebookNode) -> Vec<NotebookId> {
    let mut ids = vec![node.notebook.id.clone()];
    for child in &node.children {
        ids.extend(all_ids(child));
    }
    ids
}

#[cfg(test)]
mod tests {
    use bourbon_core::{Notebook, Timestamp};

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

    fn listed(nodes: &[NotebookNode], expanded: &HashSet<NotebookId>) -> Vec<(u16, String, bool)> {
        visible_notebooks(nodes, expanded)
            .into_iter()
            .map(|row| (row.depth, row.node.notebook.name.clone(), row.is_expanded))
            .collect()
    }

    fn sample() -> Vec<NotebookNode> {
        vec![
            node(
                "A",
                vec![
                    node("A1", vec![node("A1a", Vec::new())]),
                    node("A2", Vec::new()),
                ],
            ),
            node("B", Vec::new()),
        ]
    }

    #[test]
    fn collapsed_notebooks_hide_their_children() {
        let tree = sample();
        let rows = listed(&tree, &HashSet::new());
        assert_eq!(
            rows,
            [(0, "A".to_owned(), false), (0, "B".to_owned(), false)]
        );
    }

    #[test]
    fn expanded_notebooks_show_children_depth_first_with_depths() {
        let tree = sample();
        let expanded = HashSet::from([
            tree[0].notebook.id.clone(),
            tree[0].children[0].notebook.id.clone(),
        ]);
        assert_eq!(
            listed(&tree, &expanded),
            [
                (0, "A".to_owned(), true),
                (1, "A1".to_owned(), true),
                (2, "A1a".to_owned(), false),
                (1, "A2".to_owned(), false),
                (0, "B".to_owned(), false),
            ]
        );
    }

    #[test]
    fn an_expanded_parent_does_not_reveal_the_children_of_a_collapsed_child() {
        let tree = sample();
        let expanded = HashSet::from([tree[0].notebook.id.clone()]);
        let names: Vec<_> = listed(&tree, &expanded)
            .into_iter()
            .map(|(_, name, _)| name)
            .collect();
        assert_eq!(names, ["A", "A1", "A2", "B"]);
    }

    #[test]
    fn expanded_lists_direct_notes_and_collapsed_lists_descendants() {
        assert_eq!(depth_for(true), NotebookDepth::DirectOnly);
        assert_eq!(depth_for(false), NotebookDepth::WithDescendants);
    }

    #[test]
    fn a_subtree_holds_the_notebook_and_all_its_descendants() {
        let tree = sample();
        let mut ids = subtree_ids(&tree, &tree[0].notebook.id);
        ids.sort_by_key(ToString::to_string);
        let mut expected = vec![
            tree[0].notebook.id.clone(),
            tree[0].children[0].notebook.id.clone(),
            tree[0].children[0].children[0].notebook.id.clone(),
            tree[0].children[1].notebook.id.clone(),
        ];
        expected.sort_by_key(ToString::to_string);
        assert_eq!(ids, expected);
    }

    #[test]
    fn a_nested_subtree_is_found_below_other_notebooks() {
        let tree = sample();
        let ids = subtree_ids(&tree, &tree[0].children[1].notebook.id);
        assert_eq!(ids, [tree[0].children[1].notebook.id.clone()]);
    }

    #[test]
    fn an_unknown_notebook_has_an_empty_subtree() {
        assert_eq!(subtree_ids(&sample(), &NotebookId::generate()).len(), 0);
    }
}
