#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerKind {
    Bullet,
    Ordered { number: usize, digits: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListMarker {
    pub kind: MarkerKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskBox {
    pub start: usize,
    pub end: usize,
    pub checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinePrefix {
    pub indent_end: usize,
    pub quote_end: usize,
    pub marker: Option<ListMarker>,
    pub task: Option<TaskBox>,
    pub content_start: usize,
}

impl LinePrefix {
    pub fn parse(line: &str) -> Self {
        let indent_end = line.len() - line.trim_start_matches([' ', '\t']).len();
        let quote_end = skip_quotes(line, indent_end);
        let marker = parse_marker(line, quote_end);
        let task = marker.and_then(|marker| parse_task(line, marker.end));
        let content_start = task
            .map(|task| task.end)
            .or(marker.map(|marker| marker.end))
            .unwrap_or(quote_end);
        Self {
            indent_end,
            quote_end,
            marker,
            task,
            content_start,
        }
    }

    pub fn has_quote(&self) -> bool {
        self.quote_end > self.indent_end
    }

    pub fn is_list_item(&self) -> bool {
        self.marker.is_some() && !self.has_quote()
    }
}

fn skip_quotes(line: &str, from: usize) -> usize {
    let mut rest = &line[from..];
    while let Some(after_quote) = rest.strip_prefix('>') {
        rest = after_quote.strip_prefix(' ').unwrap_or(after_quote);
    }
    line.len() - rest.len()
}

fn spaces_len(line: &str, from: usize) -> usize {
    line[from..].len() - line[from..].trim_start_matches(' ').len()
}

fn parse_marker(line: &str, from: usize) -> Option<ListMarker> {
    let rest = &line[from..];
    let first = rest.chars().next()?;
    let (kind, symbol_len) = if matches!(first, '-' | '*' | '+') {
        (MarkerKind::Bullet, 1)
    } else {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        let delimiter = rest[digits..].chars().next()?;
        if digits == 0 || digits > 9 || !matches!(delimiter, '.' | ')') {
            return None;
        }
        let number = rest[..digits].parse().ok()?;
        (MarkerKind::Ordered { number, digits }, digits + 1)
    };
    let spaces = spaces_len(line, from + symbol_len);
    (spaces > 0).then_some(ListMarker {
        kind,
        start: from,
        end: from + symbol_len + spaces,
    })
}

fn parse_task(line: &str, from: usize) -> Option<TaskBox> {
    let rest = &line[from..];
    let checked = match rest.get(..3)? {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    let trailing = rest[3..].starts_with(' ');
    if rest.len() > 3 && !trailing {
        return None;
    }
    Some(TaskBox {
        start: from,
        end: from + 3 + usize::from(trailing),
        checked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_line_has_no_markup() {
        let prefix = LinePrefix::parse("hello");
        assert_eq!(prefix.indent_end, 0);
        assert_eq!(prefix.quote_end, 0);
        assert!(prefix.marker.is_none());
        assert!(prefix.task.is_none());
        assert_eq!(prefix.content_start, 0);
        assert!(!prefix.is_list_item());
    }

    #[test]
    fn indentation_is_measured_in_bytes() {
        assert_eq!(LinePrefix::parse("   x").indent_end, 3);
        assert_eq!(LinePrefix::parse("\tx").indent_end, 1);
    }

    #[test]
    fn bullets_use_dash_star_plus_followed_by_spaces() {
        for line in ["- a", "* a", "+ a"] {
            let prefix = LinePrefix::parse(line);
            let marker = prefix.marker.expect("bullet");
            assert_eq!(marker.kind, MarkerKind::Bullet);
            assert_eq!((marker.start, marker.end), (0, 2));
            assert_eq!(prefix.content_start, 2);
            assert!(prefix.is_list_item());
        }
        assert_eq!(LinePrefix::parse("-   a").marker.map(|m| m.end), Some(4));
    }

    #[test]
    fn dash_without_space_is_not_a_marker() {
        assert!(LinePrefix::parse("-a").marker.is_none());
        assert!(LinePrefix::parse("**bold**").marker.is_none());
        assert!(LinePrefix::parse("---").marker.is_none());
        assert!(LinePrefix::parse("-").marker.is_none());
    }

    #[test]
    fn empty_bullet_item_keeps_its_trailing_space() {
        let prefix = LinePrefix::parse("- ");
        assert_eq!(prefix.content_start, 2);
    }

    #[test]
    fn ordered_markers_capture_the_number_and_digits() {
        let prefix = LinePrefix::parse("  12. text");
        let marker = prefix.marker.expect("ordered");
        assert_eq!(
            marker.kind,
            MarkerKind::Ordered {
                number: 12,
                digits: 2
            }
        );
        assert_eq!((marker.start, marker.end), (2, 6));
        let paren = LinePrefix::parse("3) x");
        assert_eq!(
            paren.marker.map(|m| m.kind),
            Some(MarkerKind::Ordered {
                number: 3,
                digits: 1
            })
        );
    }

    #[test]
    fn nine_digit_numbers_are_still_markers() {
        assert!(LinePrefix::parse("123456789. x").marker.is_some());
    }

    #[test]
    fn invalid_ordered_markers_are_rejected() {
        assert!(LinePrefix::parse("1.x").marker.is_none());
        assert!(LinePrefix::parse("a. x").marker.is_none());
        assert!(LinePrefix::parse("1234567890. x").marker.is_none());
        assert!(LinePrefix::parse("1 x").marker.is_none());
        assert!(LinePrefix::parse(". x").marker.is_none());
    }

    #[test]
    fn quotes_nest_and_precede_markers() {
        let prefix = LinePrefix::parse("> > - x");
        assert_eq!(prefix.quote_end, 4);
        assert!(prefix.has_quote());
        assert!(!prefix.is_list_item());
        assert_eq!(prefix.marker.map(|m| (m.start, m.end)), Some((4, 6)));
        assert_eq!(LinePrefix::parse(">x").quote_end, 1);
        assert_eq!(LinePrefix::parse("  > x").quote_end, 4);
    }

    #[test]
    fn tasks_follow_the_marker() {
        let open = LinePrefix::parse("- [ ] todo");
        let task = open.task.expect("task");
        assert!(!task.checked);
        assert_eq!((task.start, task.end), (2, 6));
        assert_eq!(open.content_start, 6);
        assert!(LinePrefix::parse("- [x] done").task.expect("task").checked);
        assert!(LinePrefix::parse("- [X] done").task.expect("task").checked);
    }

    #[test]
    fn task_may_end_the_line() {
        let prefix = LinePrefix::parse("- [ ]");
        assert_eq!(prefix.task.map(|t| t.end), Some(5));
    }

    #[test]
    fn brackets_that_are_not_tasks_are_content() {
        assert!(LinePrefix::parse("- [y] x").task.is_none());
        assert!(LinePrefix::parse("- [ ]x").task.is_none());
        assert!(LinePrefix::parse("- [ x").task.is_none());
        assert!(LinePrefix::parse("[ ] x").task.is_none());
        assert!(LinePrefix::parse("- é").task.is_none());
    }
}
