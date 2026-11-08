use bourbon_markdown::{
    Anchor, PreviewDoc, highlight, line_decorations, line_for_preview_offset,
    preview_offset_for_line, task_lines,
};
use proptest::prelude::*;

fn markdown_like() -> impl Strategy<Value = String> {
    prop_oneof![
        ".{0,200}",
        "[a-z#*_~`>\\[\\]()!|:$<\\-+.0-9 \\n\\t\\\\é✓]{0,300}",
    ]
}

fn anchors() -> impl Strategy<Value = Vec<Anchor>> {
    prop::collection::vec((0usize..4, 0usize..3, 0.0f32..60.0, 1.0f32..80.0), 0..12).prop_map(
        |steps| {
            let mut line = 0;
            let mut top = 0.0;
            steps
                .into_iter()
                .map(|(line_step, extra_lines, top_step, height)| {
                    line += line_step;
                    top += top_step;
                    Anchor {
                        line_start: line,
                        line_end: line + extra_lines,
                        top,
                        height,
                    }
                })
                .collect()
        },
    )
}

proptest! {
    #[test]
    fn decorations_cover_existing_lines_in_order(source in markdown_like()) {
        let line_count = source.split('\n').count();
        for decoration in line_decorations(&source) {
            prop_assert!(decoration.lines.start() <= decoration.lines.end());
            prop_assert!(*decoration.lines.end() < line_count);
        }
    }

    #[test]
    fn highlight_spans_are_sorted_disjoint_and_on_char_boundaries(source in markdown_like()) {
        let spans = highlight(&source);
        let mut previous_end = 0;
        for span in spans {
            prop_assert!(span.range.start < span.range.end);
            prop_assert!(span.range.start >= previous_end);
            prop_assert!(span.range.end <= source.len());
            prop_assert!(source.is_char_boundary(span.range.start));
            prop_assert!(source.is_char_boundary(span.range.end));
            previous_end = span.range.end;
        }
    }

    #[test]
    fn parsing_never_panics_and_lines_stay_in_bounds(source in markdown_like()) {
        let line_count = source.matches('\n').count() + 1;
        let doc = PreviewDoc::parse(&source);
        for block in &doc.blocks {
            prop_assert!(block.source_lines.0 <= block.source_lines.1);
            prop_assert!(block.source_lines.1 < line_count);
        }
        for (line, _) in task_lines(&source) {
            prop_assert!(line < line_count);
        }
    }

    #[test]
    fn preview_offset_is_monotonic_in_line(
        anchors in anchors(),
        first in 0.0f32..60.0,
        gap in 0.0f32..30.0,
    ) {
        let low = preview_offset_for_line(&anchors, first);
        let high = preview_offset_for_line(&anchors, first + gap);
        prop_assert!(high >= low - 1e-3, "{low} then {high}");
    }

    #[test]
    fn line_is_monotonic_in_preview_offset(
        anchors in anchors(),
        first in 0.0f32..800.0,
        gap in 0.0f32..400.0,
    ) {
        let low = line_for_preview_offset(&anchors, first);
        let high = line_for_preview_offset(&anchors, first + gap);
        prop_assert!(high >= low - 1e-3, "{low} then {high}");
    }
}
