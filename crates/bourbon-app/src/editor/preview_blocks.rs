use super::tasks::parse_task_line;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreviewBlock {
    Markdown(String),
    Task {
        line: usize,
        indent: usize,
        checked: bool,
        label: String,
    },
}

pub(crate) fn preview_blocks(body: &str) -> Vec<PreviewBlock> {
    let mut blocks = Vec::new();
    let mut pending: Vec<&str> = Vec::new();
    let mut inside_fence = false;
    for (line_index, line) in body.split('\n').enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            inside_fence = !inside_fence;
        }
        let task = if inside_fence {
            None
        } else {
            parse_task_line(line)
        };
        match task {
            Some(task) => {
                flush_markdown(&mut pending, &mut blocks);
                blocks.push(PreviewBlock::Task {
                    line: line_index,
                    indent: task.indent,
                    checked: task.checked,
                    label: task.label.to_owned(),
                });
            }
            None => pending.push(line),
        }
    }
    flush_markdown(&mut pending, &mut blocks);
    blocks
}

fn flush_markdown(pending: &mut Vec<&str>, blocks: &mut Vec<PreviewBlock>) {
    if pending.iter().any(|line| !line.trim().is_empty()) {
        blocks.push(PreviewBlock::Markdown(pending.join("\n")));
    }
    pending.clear();
}

#[cfg(test)]
mod tests {
    use super::{PreviewBlock, preview_blocks};

    fn markdown(text: &str) -> PreviewBlock {
        PreviewBlock::Markdown(text.to_owned())
    }

    fn task(line: usize, indent: usize, checked: bool, label: &str) -> PreviewBlock {
        PreviewBlock::Task {
            line,
            indent,
            checked,
            label: label.to_owned(),
        }
    }

    #[test]
    fn a_body_without_tasks_is_one_markdown_block() {
        assert_eq!(
            preview_blocks("# Title\n\ntext"),
            vec![markdown("# Title\n\ntext")]
        );
    }

    #[test]
    fn an_empty_body_has_no_blocks() {
        assert_eq!(preview_blocks(""), Vec::new());
    }

    #[test]
    fn tasks_split_the_markdown_around_them_and_keep_their_source_line() {
        let body = "intro\n- [ ] a\n- [x] b\n\n| h |\n|---|\n| c |";
        assert_eq!(
            preview_blocks(body),
            vec![
                markdown("intro"),
                task(1, 0, false, "a"),
                task(2, 0, true, "b"),
                markdown("\n| h |\n|---|\n| c |"),
            ]
        );
    }

    #[test]
    fn nested_tasks_carry_their_indent() {
        assert_eq!(
            preview_blocks("- [ ] a\n    - [x] b"),
            vec![task(0, 0, false, "a"), task(1, 4, true, "b")]
        );
    }

    #[test]
    fn task_looking_lines_inside_a_code_fence_stay_code() {
        let body = "```\n- [ ] not a task\n```\n- [ ] real";
        assert_eq!(
            preview_blocks(body),
            vec![
                markdown("```\n- [ ] not a task\n```"),
                task(3, 0, false, "real")
            ]
        );
    }

    #[test]
    fn a_tilde_fence_also_protects_its_lines() {
        let body = "~~~\n- [x] code\n~~~";
        assert_eq!(preview_blocks(body), vec![markdown(body)]);
    }

    #[test]
    fn blank_lines_alone_between_tasks_produce_no_block() {
        assert_eq!(
            preview_blocks("- [ ] a\n\n- [ ] b"),
            vec![task(0, 0, false, "a"), task(2, 0, false, "b")]
        );
    }
}
