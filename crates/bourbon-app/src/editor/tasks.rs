use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextEdit {
    pub(crate) range: Range<usize>,
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskLine<'a> {
    pub(crate) indent: usize,
    pub(crate) checked: bool,
    pub(crate) label: &'a str,
    pub(crate) mark_offset: usize,
}

const BULLETS: [char; 3] = ['-', '*', '+'];

pub(crate) fn parse_task_line(line: &str) -> Option<TaskLine<'_>> {
    let (indent, after_marker) = bullet_prefix(line)?;
    let mark_offset = line.len() - after_marker.len() + 1;
    let mut chars = after_marker.chars();
    if chars.next()? != '[' {
        return None;
    }
    let checked = match chars.next()? {
        ' ' => false,
        'x' | 'X' => true,
        _ => return None,
    };
    if chars.next()? != ']' {
        return None;
    }
    let label = match chars.as_str() {
        "" => "",
        rest => rest.strip_prefix(' ')?,
    };
    Some(TaskLine {
        indent,
        checked,
        label,
        mark_offset,
    })
}

fn bullet_prefix(line: &str) -> Option<(usize, &str)> {
    let content = line.trim_start_matches([' ', '\t']);
    let indent = line.len() - content.len();
    let after_bullet = content.strip_prefix(BULLETS)?;
    let after_space = after_bullet.strip_prefix(' ')?;
    Some((indent, after_space))
}

pub(crate) fn toggle_task_on_line(body: &str, line_index: usize) -> Option<TextEdit> {
    let (line_start, line) = line_at_index(body, line_index)?;
    toggle_task_in_line(line_start, line)
}

pub(crate) fn toggle_task_at_offset(body: &str, offset: usize) -> Option<TextEdit> {
    let offset = offset.min(body.len());
    let line_start = body[..offset].rfind('\n').map_or(0, |newline| newline + 1);
    let line_end = body[line_start..]
        .find('\n')
        .map_or(body.len(), |newline| line_start + newline);
    toggle_task_in_line(line_start, &body[line_start..line_end])
}

fn line_at_index(body: &str, line_index: usize) -> Option<(usize, &str)> {
    let mut start = 0;
    for (index, line) in body.split('\n').enumerate() {
        if index == line_index {
            return Some((start, line));
        }
        start += line.len() + 1;
    }
    None
}

fn toggle_task_in_line(line_start: usize, line: &str) -> Option<TextEdit> {
    if let Some(task) = parse_task_line(line) {
        let mark = line_start + task.mark_offset;
        let text = if task.checked { " " } else { "x" };
        return Some(TextEdit {
            range: mark..mark + 1,
            text: text.to_owned(),
        });
    }
    let (_, after_marker) = bullet_prefix(line)?;
    let insertion = line_start + line.len() - after_marker.len();
    Some(TextEdit {
        range: insertion..insertion,
        text: "[ ] ".to_owned(),
    })
}

#[cfg(test)]
pub(crate) fn apply(body: &str, edit: &TextEdit) -> String {
    let mut result = String::with_capacity(body.len() + edit.text.len());
    result.push_str(&body[..edit.range.start]);
    result.push_str(&edit.text);
    result.push_str(&body[edit.range.end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::{TaskLine, apply, parse_task_line, toggle_task_at_offset, toggle_task_on_line};

    fn toggled_line(body: &str, line: usize) -> Option<String> {
        toggle_task_on_line(body, line).map(|edit| apply(body, &edit))
    }

    fn toggled_at(body: &str, offset: usize) -> Option<String> {
        toggle_task_at_offset(body, offset).map(|edit| apply(body, &edit))
    }

    #[test]
    fn an_open_task_becomes_done() {
        assert_eq!(toggled_line("- [ ] milk", 0).as_deref(), Some("- [x] milk"));
    }

    #[test]
    fn a_done_task_reopens_for_both_cases_of_x() {
        assert_eq!(toggled_line("- [x] milk", 0).as_deref(), Some("- [ ] milk"));
        assert_eq!(toggled_line("- [X] milk", 0).as_deref(), Some("- [ ] milk"));
    }

    #[test]
    fn only_the_requested_line_changes() {
        let body = "intro\n- [ ] a\n- [ ] b\noutro";
        assert_eq!(
            toggled_line(body, 2).as_deref(),
            Some("intro\n- [ ] a\n- [x] b\noutro")
        );
    }

    #[test]
    fn indented_and_alternative_bullets_are_tasks() {
        assert_eq!(toggled_line("  * [ ] a", 0).as_deref(), Some("  * [x] a"));
        assert_eq!(toggled_line("\t+ [x] a", 0).as_deref(), Some("\t+ [ ] a"));
    }

    #[test]
    fn an_empty_task_label_still_toggles() {
        assert_eq!(toggled_line("- [ ]", 0).as_deref(), Some("- [x]"));
    }

    #[test]
    fn a_plain_list_line_becomes_an_open_task() {
        assert_eq!(toggled_line("- milk", 0).as_deref(), Some("- [ ] milk"));
        assert_eq!(toggled_line("  * milk", 0).as_deref(), Some("  * [ ] milk"));
    }

    #[test]
    fn lines_that_are_not_list_items_are_left_alone() {
        assert_eq!(toggled_line("milk", 0), None);
        assert_eq!(toggled_line("-milk", 0), None);
        assert_eq!(toggled_line("a\nb", 5), None);
    }

    #[test]
    fn brackets_that_are_not_a_checkbox_are_content_of_a_plain_list_item() {
        assert_eq!(
            toggled_line("- [y] milk", 0).as_deref(),
            Some("- [ ] [y] milk")
        );
        assert_eq!(
            toggled_line("- [x]milk", 0).as_deref(),
            Some("- [ ] [x]milk")
        );
    }

    #[test]
    fn the_cursor_line_is_found_from_any_offset_in_it() {
        let body = "x\n- [ ] a\ny";
        assert_eq!(toggled_at(body, 2).as_deref(), Some("x\n- [x] a\ny"));
        assert_eq!(toggled_at(body, 6).as_deref(), Some("x\n- [x] a\ny"));
        assert_eq!(toggled_at(body, 9).as_deref(), Some("x\n- [x] a\ny"));
        assert_eq!(toggled_at(body, 10), None);
    }

    #[test]
    fn an_offset_past_the_end_uses_the_last_line() {
        assert_eq!(toggled_at("- [ ] a", 99).as_deref(), Some("- [x] a"));
    }

    #[test]
    fn multibyte_text_keeps_byte_offsets_right() {
        let body = "ação\n- [ ] pão";
        assert_eq!(toggled_at(body, 8).as_deref(), Some("ação\n- [x] pão"));
    }

    #[test]
    fn parsing_reports_indent_state_label_and_mark_position() {
        assert_eq!(
            parse_task_line("  - [x] buy milk"),
            Some(TaskLine {
                indent: 2,
                checked: true,
                label: "buy milk",
                mark_offset: 5
            })
        );
        assert_eq!(parse_task_line("- plain"), None);
        assert_eq!(parse_task_line("- [ ]x"), None);
    }
}
