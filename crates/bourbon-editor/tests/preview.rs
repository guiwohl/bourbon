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
        preview.set_theme(PreviewTheme::dark(), cx)
    });
    let dark = preview.read_with(cx, |preview, _| preview.theme().clone());
    assert_eq!(dark, PreviewTheme::dark());
}
