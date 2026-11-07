#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskProgress {
    pub done: usize,
    pub total: usize,
}

const BULLETS: [char; 3] = ['-', '*', '+'];

impl TaskProgress {
    #[must_use]
    pub fn of_body(body: &str) -> Option<Self> {
        let mut inside_fence = false;
        let mut progress = Self { done: 0, total: 0 };
        for line in body.lines().map(str::trim_start) {
            if line.starts_with("```") || line.starts_with("~~~") {
                inside_fence = !inside_fence;
            } else if !inside_fence {
                match task_state(line) {
                    Some(true) => {
                        progress.done += 1;
                        progress.total += 1;
                    }
                    Some(false) => progress.total += 1,
                    None => {}
                }
            }
        }
        (progress.total > 0).then_some(progress)
    }
}

fn task_state(line: &str) -> Option<bool> {
    let after_bullet = line.strip_prefix(BULLETS)?.strip_prefix(' ')?;
    let mark = after_bullet.strip_prefix('[')?;
    let mut characters = mark.chars();
    let state = characters.next()?;
    if characters.next()? != ']' {
        return None;
    }
    match state {
        ' ' => Some(false),
        'x' | 'X' => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(done: usize, total: usize) -> TaskProgress {
        TaskProgress { done, total }
    }

    #[test]
    fn a_body_without_tasks_has_no_progress() {
        assert_eq!(TaskProgress::of_body("plain\n- item\n* other"), None);
        assert_eq!(TaskProgress::of_body(""), None);
    }

    #[test]
    fn open_and_checked_tasks_are_counted_with_both_bullets() {
        let body = "- [ ] a\n- [x] b\n* [ ] c\n* [X] d\n+ [ ] e";
        assert_eq!(TaskProgress::of_body(body), Some(progress(2, 5)));
    }

    #[test]
    fn indented_tasks_count_too() {
        assert_eq!(
            TaskProgress::of_body("- [x] a\n    - [ ] b"),
            Some(progress(1, 2))
        );
    }

    #[test]
    fn tasks_inside_code_fences_are_ignored() {
        let body = "- [x] real\n```\n- [ ] fake\n- [x] fake\n```\n~~~\n- [ ] fake\n~~~\n- [ ] real";
        assert_eq!(TaskProgress::of_body(body), Some(progress(1, 2)));
    }

    #[test]
    fn malformed_marks_are_not_tasks() {
        assert_eq!(
            TaskProgress::of_body("- [] a\n- [y] b\n-[ ] c\n- [ x] d"),
            None
        );
    }
}
