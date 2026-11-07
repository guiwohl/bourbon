use crate::markup::LinePrefix;
use crate::state::{EditorState, Transaction};

use super::{line_change, line_edits_transaction, selected_lines};

pub fn toggle_task(state: &EditorState) -> Option<Transaction> {
    let changes = selected_lines(&state.text, &state.selection)
        .into_iter()
        .filter_map(|line| {
            let line_text = state.text.line_text(line);
            let toggled = toggled_line(&line_text);
            line_change(state.text.line_start(line), &line_text, &toggled)
        })
        .collect();
    line_edits_transaction(state, changes)
}

fn toggled_line(line: &str) -> String {
    let prefix = LinePrefix::parse(line);
    if let Some(task) = prefix.task {
        let mark = if task.checked { ' ' } else { 'x' };
        return format!("{}{mark}{}", &line[..=task.start], &line[task.start + 2..]);
    }
    if let Some(marker) = prefix.marker {
        return format!("{}[ ] {}", &line[..marker.end], &line[marker.end..]);
    }
    format!(
        "{}- [ ] {}",
        &line[..prefix.quote_end],
        &line[prefix.quote_end..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::expect;

    #[test]
    fn unchecked_becomes_checked_and_back() {
        expect("- [ ] a|", toggle_task, "- [x] a|");
        expect("- [x] a|", toggle_task, "- [ ] a|");
        expect("- [X] a|", toggle_task, "- [ ] a|");
    }

    #[test]
    fn works_for_ordered_and_other_bullets() {
        expect("1. [ ] a|", toggle_task, "1. [x] a|");
        expect("* [ ] a|", toggle_task, "* [x] a|");
    }

    #[test]
    fn plain_list_item_becomes_a_task() {
        expect("- a|", toggle_task, "- [ ] a|");
        expect("  1. a|", toggle_task, "  1. [ ] a|");
        expect("- |", toggle_task, "- [ ] |");
    }

    #[test]
    fn non_list_line_becomes_a_bullet_task() {
        expect("abc|", toggle_task, "- [ ] abc|");
        expect("  abc|", toggle_task, "  - [ ] abc|");
        expect("> abc|", toggle_task, "> - [ ] abc|");
    }

    #[test]
    fn caret_before_the_inserted_marker_moves_past_it() {
        expect("|abc", toggle_task, "- [ ] |abc");
    }

    #[test]
    fn every_selected_line_toggles_on_its_own() {
        expect(
            "«- [ ] a\n- b\n- [x] c»",
            toggle_task,
            "«- [x] a\n- [ ] b\n- [ ] c»",
        );
    }

    #[test]
    fn empty_document_gets_a_task() {
        expect("|", toggle_task, "- [ ] |");
    }

    #[test]
    fn checked_toggle_is_a_single_character_change() {
        let state = crate::testing::state("- [ ] a|");
        let transaction = toggle_task(&state).expect("toggle");
        assert_eq!(transaction.changes.changes().len(), 1);
        assert_eq!(transaction.changes.changes()[0].range, 3..4);
    }
}
