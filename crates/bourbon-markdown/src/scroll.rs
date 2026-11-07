#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub line_start: usize,
    pub line_end: usize,
    pub top: f32,
    pub height: f32,
}

const PREVIEW_LEAD: f32 = 10.0;

#[derive(Clone, Copy)]
struct Stop {
    line: f32,
    top: f32,
}

const DOCUMENT_START: Stop = Stop {
    line: 0.0,
    top: 0.0,
};

fn start_stop(anchor: &Anchor) -> Stop {
    Stop {
        line: lossy(anchor.line_start),
        top: anchor.top,
    }
}

fn end_stop(anchor: &Anchor) -> Stop {
    Stop {
        line: lossy(anchor.line_end + 1),
        top: anchor.top + anchor.height,
    }
}

#[allow(clippy::cast_precision_loss)]
fn lossy(line: usize) -> f32 {
    line as f32
}

fn progress(value: f32, from: f32, to: f32) -> f32 {
    if to > from {
        ((value - from) / (to - from)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn preview_offset_for_line(anchors: &[Anchor], line: f32) -> f32 {
    let whole_line = line.max(0.0).floor() as usize;
    let above = anchors
        .iter()
        .filter(|anchor| anchor.line_start <= whole_line)
        .reduce(|best, anchor| {
            if anchor.line_start > best.line_start {
                anchor
            } else {
                best
            }
        })
        .map_or(DOCUMENT_START, start_stop);
    let below = anchors
        .iter()
        .find(|anchor| anchor.line_start > whole_line)
        .map(start_stop)
        .or_else(|| anchors.last().map(end_stop))
        .unwrap_or(above);
    let t = progress(line, above.line, below.line);
    (lerp(above.top, below.top, t) - PREVIEW_LEAD).max(0.0)
}

#[must_use]
pub fn line_for_preview_offset(anchors: &[Anchor], offset: f32) -> f32 {
    let above_index = anchors.iter().rposition(|anchor| anchor.top <= offset);
    let above = above_index.map_or(DOCUMENT_START, |index| start_stop(&anchors[index]));
    let below = anchors
        .get(above_index.map_or(0, |index| index + 1))
        .map(start_stop)
        .or_else(|| anchors.last().map(end_stop))
        .unwrap_or(above);
    lerp(
        above.line,
        below.line,
        progress(offset, above.top, below.top),
    )
}

#[cfg(test)]
mod tests {
    use super::{Anchor, line_for_preview_offset, preview_offset_for_line};

    fn anchor(line_start: usize, line_end: usize, top: f32, height: f32) -> Anchor {
        Anchor {
            line_start,
            line_end,
            top,
            height,
        }
    }

    fn sample() -> Vec<Anchor> {
        vec![
            anchor(0, 1, 0.0, 50.0),
            anchor(4, 5, 100.0, 40.0),
            anchor(10, 11, 300.0, 60.0),
        ]
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-3, "{actual} != {expected}");
    }

    #[test]
    fn preview_offset_interpolates_between_anchors_with_lead() {
        assert_close(preview_offset_for_line(&sample(), 7.0), 200.0 - 10.0);
        assert_close(
            preview_offset_for_line(&sample(), 5.0),
            100.0 + 200.0 / 6.0 - 10.0,
        );
    }

    #[test]
    fn preview_offset_at_an_anchor_line_is_its_top_minus_lead() {
        assert_close(preview_offset_for_line(&sample(), 4.0), 90.0);
        assert_close(preview_offset_for_line(&sample(), 10.0), 290.0);
    }

    #[test]
    fn preview_offset_uses_the_fractional_part_of_the_line() {
        assert_close(
            preview_offset_for_line(&sample(), 4.5),
            100.0 + 200.0 * 0.5 / 6.0 - 10.0,
        );
    }

    #[test]
    fn preview_offset_never_goes_negative() {
        assert_close(preview_offset_for_line(&sample(), 0.0), 0.0);
        assert_close(
            preview_offset_for_line(&sample(), 0.5),
            100.0 * 0.5 / 4.0 - 10.0,
        );
    }

    #[test]
    fn preview_offset_after_last_anchor_interpolates_to_its_bottom() {
        assert_close(preview_offset_for_line(&sample(), 11.0), 330.0 - 10.0);
        assert_close(preview_offset_for_line(&sample(), 12.0), 360.0 - 10.0);
        assert_close(preview_offset_for_line(&sample(), 50.0), 360.0 - 10.0);
    }

    #[test]
    fn preview_offset_before_first_anchor_starts_from_document_top() {
        let anchors = vec![anchor(4, 5, 100.0, 40.0), anchor(8, 9, 200.0, 40.0)];
        assert_close(preview_offset_for_line(&anchors, 2.0), 50.0 - 10.0);
    }

    #[test]
    fn preview_offset_uses_first_anchor_sharing_a_line() {
        let anchors = vec![
            anchor(2, 6, 10.0, 80.0),
            anchor(2, 3, 20.0, 20.0),
            anchor(6, 7, 100.0, 20.0),
        ];
        assert_close(preview_offset_for_line(&anchors, 4.0), 55.0 - 10.0);
    }

    #[test]
    fn preview_offset_without_anchors_is_zero() {
        assert_close(preview_offset_for_line(&[], 5.0), 0.0);
    }

    #[test]
    fn line_interpolates_between_surrounding_anchors() {
        assert_close(line_for_preview_offset(&sample(), 200.0), 7.0);
        assert_close(line_for_preview_offset(&sample(), 125.0), 4.75);
    }

    #[test]
    fn line_at_an_anchor_top_is_its_start_line() {
        assert_close(line_for_preview_offset(&sample(), 100.0), 4.0);
        assert_close(line_for_preview_offset(&sample(), 0.0), 0.0);
    }

    #[test]
    fn line_after_last_anchor_uses_its_bottom_and_end_plus_one() {
        assert_close(line_for_preview_offset(&sample(), 330.0), 10.0 + 0.5 * 2.0);
        assert_close(line_for_preview_offset(&sample(), 360.0), 12.0);
        assert_close(line_for_preview_offset(&sample(), 900.0), 12.0);
    }

    #[test]
    fn line_before_first_anchor_starts_from_document_top() {
        let anchors = vec![anchor(4, 5, 100.0, 40.0)];
        assert_close(line_for_preview_offset(&anchors, 50.0), 2.0);
        assert_close(line_for_preview_offset(&anchors, -20.0), 0.0);
    }

    #[test]
    fn zero_height_last_anchor_does_not_divide_by_zero() {
        let anchors = vec![anchor(3, 3, 50.0, 0.0)];
        assert_close(line_for_preview_offset(&anchors, 60.0), 3.0);
        assert_close(preview_offset_for_line(&anchors, 9.0), 40.0);
    }

    #[test]
    fn line_without_anchors_is_zero() {
        assert_close(line_for_preview_offset(&[], 40.0), 0.0);
    }
}
