const BULLETS: [&str; 3] = ["- ", "* ", "+ "];
const TASK_MARKS: [&str; 3] = ["[ ] ", "[x] ", "[X] "];
const DROPPED_RUNS: [&str; 5] = ["**", "__", "~~", "`", "*"];

pub(crate) fn strip_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return String::new();
    }
    let content = strip_block_marker(trimmed);
    let mut text = without_links(content);
    for run in DROPPED_RUNS {
        text = text.replace(run, "");
    }
    text.trim().to_owned()
}

fn strip_block_marker(line: &str) -> &str {
    let line = line.trim_start_matches('#').trim_start();
    let line = line.strip_prefix('>').unwrap_or(line).trim_start();
    let line = BULLETS
        .iter()
        .find_map(|bullet| line.strip_prefix(bullet))
        .unwrap_or_else(|| ordered_remainder(line));
    TASK_MARKS
        .iter()
        .find_map(|mark| line.strip_prefix(mark))
        .unwrap_or(line)
}

fn ordered_remainder(line: &str) -> &str {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return line;
    }
    line[digits..]
        .strip_prefix(". ")
        .or_else(|| line[digits..].strip_prefix(") "))
        .unwrap_or(line)
}

fn without_links(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some((before, after_bracket)) = rest.split_once('[') {
        if let Some((label, remainder)) = link_after_bracket(after_bracket) {
            output.push_str(before.trim_end_matches('!'));
            output.push_str(label);
            rest = remainder;
        } else {
            output.push_str(before);
            output.push('[');
            rest = after_bracket;
        }
    }
    output.push_str(rest);
    output
}

fn link_after_bracket(text: &str) -> Option<(&str, &str)> {
    let (label, after_label) = text.split_once("](")?;
    if label.contains('[') {
        return None;
    }
    let (_, remainder) = after_label.split_once(')')?;
    Some((label, remainder))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_lose_their_hashes() {
        assert_eq!(strip_line("## Release notes"), "Release notes");
    }

    #[test]
    fn list_markers_and_task_boxes_are_dropped() {
        assert_eq!(strip_line("- item"), "item");
        assert_eq!(strip_line("* item"), "item");
        assert_eq!(strip_line("3. third"), "third");
        assert_eq!(strip_line("- [ ] todo"), "todo");
        assert_eq!(strip_line("- [x] done"), "done");
    }

    #[test]
    fn quotes_lose_their_marker() {
        assert_eq!(strip_line("> quoted"), "quoted");
    }

    #[test]
    fn emphasis_and_code_marks_are_removed() {
        assert_eq!(
            strip_line("a **bold** and *it* and `code` and ~~gone~~"),
            "a bold and it and code and gone"
        );
    }

    #[test]
    fn links_and_images_keep_only_their_label() {
        assert_eq!(
            strip_line("see [the docs](http://x.y/z) now"),
            "see the docs now"
        );
        assert_eq!(strip_line("![diagram](a.png) after"), "diagram after");
    }

    #[test]
    fn brackets_that_are_not_links_stay() {
        assert_eq!(strip_line("array[0] and [ok"), "array[0] and [ok");
    }

    #[test]
    fn code_fences_become_empty() {
        assert_eq!(strip_line("```rust"), "");
        assert_eq!(strip_line("~~~"), "");
    }

    #[test]
    fn a_hyphen_without_space_is_text() {
        assert_eq!(strip_line("-5 degrees"), "-5 degrees");
    }
}
