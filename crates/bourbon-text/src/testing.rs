use crate::selection::{Selection, SelectionRange};
use crate::state::{EditorState, Transaction};
use crate::text::Text;

pub fn state(spec: &str) -> EditorState {
    let mut content = String::new();
    let mut ranges = Vec::new();
    let mut open: Option<(usize, bool)> = None;
    for character in spec.chars() {
        match character {
            '|' => ranges.push(SelectionRange::cursor(content.len())),
            '«' | '»' => match open.take() {
                Some((first, head_first)) => {
                    let second = content.len();
                    ranges.push(if head_first {
                        SelectionRange::new(second, first)
                    } else {
                        SelectionRange::new(first, second)
                    });
                }
                None => open = Some((content.len(), character == '»')),
            },
            other => content.push(other),
        }
    }
    EditorState::new(Text::new(&content), Selection::new(ranges, 0))
}

pub fn render(state: &EditorState) -> String {
    let mut markers: Vec<(usize, char)> = Vec::new();
    for range in state.selection.ranges() {
        if range.is_empty() {
            markers.push((range.head, '|'));
        } else if range.is_reversed() {
            markers.push((range.from(), '»'));
            markers.push((range.to(), '«'));
        } else {
            markers.push((range.from(), '«'));
            markers.push((range.to(), '»'));
        }
    }
    markers.sort_by_key(|(position, _)| *position);
    let content = state.text.to_string();
    let mut rendered = String::new();
    let mut cursor = 0;
    for (position, marker) in markers {
        rendered.push_str(&content[cursor..position]);
        rendered.push(marker);
        cursor = position;
    }
    rendered.push_str(&content[cursor..]);
    rendered
}

pub fn expect(before: &str, command: impl Fn(&EditorState) -> Option<Transaction>, after: &str) {
    let state = state(before);
    let transaction = command(&state).unwrap_or_else(|| panic!("command did nothing on {before}"));
    assert_eq!(render(&state.apply(&transaction)), after);
}

pub fn expect_nothing(before: &str, command: impl Fn(&EditorState) -> Option<Transaction>) {
    assert_eq!(command(&state(before)), None);
}
