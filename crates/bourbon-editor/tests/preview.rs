use std::cell::RefCell;
use std::rc::Rc;

use bourbon_editor::{PreviewEvent, PreviewTheme, PreviewView};
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};

fn open<'a>(
    cx: &'a mut TestAppContext,
    source: &str,
) -> (Entity<PreviewView>, &'a mut VisualTestContext) {
    let source = source.to_owned();
    let (preview, cx) =
        cx.add_window_view(move |_, cx| PreviewView::new(&source, PreviewTheme::light(), cx));
    cx.run_until_parked();
    (preview, cx)
}

fn record(
    preview: &Entity<PreviewView>,
    cx: &mut VisualTestContext,
) -> Rc<RefCell<Vec<PreviewEvent>>> {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    cx.update(|_, cx| {
        cx.subscribe(preview, move |_, event: &PreviewEvent, _| {
            sink.borrow_mut().push(event.clone());
        })
        .detach();
    });
    seen
}

#[gpui::test]
fn clicking_a_checkbox_emits_the_source_line_of_the_task(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "# Plan\n\n- [ ] first\n- [x] second\n");
    let seen = record(&preview, cx);
    let bounds = cx
        .debug_bounds("task-line-3")
        .expect("second checkbox is laid out");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert_eq!(
        seen.borrow().as_slice(),
        [PreviewEvent::ToggleTask { source_line: 3 }]
    );
}

#[gpui::test]
fn each_checkbox_reports_its_own_line(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "- [ ] first\n- [x] second\n");
    let seen = record(&preview, cx);
    let bounds = cx
        .debug_bounds("task-line-0")
        .expect("first checkbox is laid out");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert_eq!(
        seen.borrow().as_slice(),
        [PreviewEvent::ToggleTask { source_line: 0 }]
    );
}

#[gpui::test]
fn top_level_blocks_report_their_geometry(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "# Title\n\nParagraph text.\n\n- item\n");
    let blocks = preview.read_with(cx, |preview, _| preview.block_layouts().to_vec());
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].source_lines, (0, 0));
    assert_eq!(blocks[1].source_lines, (2, 2));
    assert_eq!(blocks[2].source_lines, (4, 4));
    assert!(blocks[1].top > blocks[0].top);
    assert!(blocks[2].top > blocks[1].top);
    assert!(blocks.iter().all(|block| block.height > gpui::px(0.)));
}

#[gpui::test]
fn changing_the_source_rebuilds_the_blocks(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "one\n");
    preview.update(cx, |preview, cx| preview.set_source("one\n\ntwo\n", cx));
    cx.run_until_parked();
    let count = preview.read_with(cx, |preview, _| preview.block_layouts().len());
    assert_eq!(count, 2);
}

#[gpui::test]
fn clicking_a_link_opens_it(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "[site](https://example.com)\n");
    let seen = record(&preview, cx);
    let bounds = cx
        .debug_bounds("inline-text-0")
        .expect("paragraph text is laid out");
    cx.simulate_click(
        gpui::point(bounds.left() + gpui::px(4.), bounds.center().y),
        Modifiers::none(),
    );
    assert_eq!(
        seen.borrow().as_slice(),
        [PreviewEvent::OpenLink("https://example.com".to_owned())]
    );
}

#[gpui::test]
fn clicking_a_note_link_opens_the_note(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "[budget](bourbon://note/abcdefghij12)\n");
    let seen = record(&preview, cx);
    let bounds = cx
        .debug_bounds("inline-text-0")
        .expect("paragraph text is laid out");
    cx.simulate_click(
        gpui::point(bounds.left() + gpui::px(4.), bounds.center().y),
        Modifiers::none(),
    );
    assert_eq!(
        seen.borrow().as_slice(),
        [PreviewEvent::OpenNote("abcdefghij12".to_owned())]
    );
}

#[gpui::test]
fn changing_the_theme_replaces_it(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "one\n");
    preview.update(cx, |preview, cx| {
        preview.set_theme(PreviewTheme::dark(), cx);
    });
    let dark = preview.read_with(cx, |preview, _| preview.theme().clone());
    assert_eq!(dark, PreviewTheme::dark());
}

#[gpui::test]
fn nested_lists_indent_and_use_the_next_marker(cx: &mut TestAppContext) {
    let (_preview, cx) = open(cx, "- a\n  - b\n");
    let outer = cx.debug_bounds("marker-0-0").expect("outer marker");
    let inner = cx.debug_bounds("marker-1-1").expect("inner marker");
    assert!(inner.left() > outer.left());
    let first = cx.debug_bounds("inline-text-0").expect("first text");
    let second = cx.debug_bounds("inline-text-1").expect("second text");
    assert!((f32::from(second.left() - first.left()) - 32.0).abs() < 0.5);
    assert!(second.top() > first.top());
}

#[gpui::test]
fn blocks_inside_quotes_alerts_and_footnotes_are_rendered(cx: &mut TestAppContext) {
    let (_preview, cx) = open(
        cx,
        "> quoted\n\n> [!NOTE]\n> alert body\n\nref[^1]\n\n[^1]: footnote text\n",
    );
    for selector in [
        "inline-text-0",
        "inline-text-1",
        "inline-text-2",
        "inline-text-3",
    ] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
}

#[gpui::test]
fn the_rule_is_a_thin_block_between_wide_gaps(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "a\n\n---\n\nb\n");
    let blocks = preview.read_with(cx, |preview, _| preview.block_layouts().to_vec());
    assert_eq!(blocks.len(), 3);
    assert!((f32::from(blocks[1].height) - 4.0).abs() < 0.05);
    let gap_before = blocks[1].top - (blocks[0].top + blocks[0].height);
    let gap_after = blocks[2].top - (blocks[1].top + blocks[1].height);
    assert!((f32::from(gap_before) - 24.0).abs() < 0.05);
    assert!((f32::from(gap_after) - 24.0).abs() < 0.05);
}

#[gpui::test]
fn headings_are_sized_by_level(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, "# one\n\n### three\n");
    let blocks = preview.read_with(cx, |preview, _| preview.block_layouts().to_vec());
    assert!((f32::from(blocks[0].height) - (40.0 + 9.6 + 2.0)).abs() < 0.1);
    assert!((f32::from(blocks[1].height) - 25.0).abs() < 0.1);
}
