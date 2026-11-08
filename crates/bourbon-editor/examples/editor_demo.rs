use std::borrow::Cow;

use bourbon_editor::{
    Editor, EditorEvent, EditorTheme, PreviewEvent, PreviewTheme, PreviewView, bind_editor_keys,
    register_editor_fonts,
};
use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_platform::application;

actions!(editor_demo, [ToggleTheme, ToggleLineNumbers]);

const NOTE: &str = "\
# App hangs when pasting a 5,000-row table

Pasting a large table into a note freezes the editor. See [[perf budget]] and [the budget](bourbon://note/abcdefghij12).

## Repro

1. Copy 5,000 rows from a spreadsheet.
2. Open a **new** note.
3. Paste with `Ctrl+V`.

## Environment

- Linux, Wayland
- *Release* build
- ~~debug~~ build

## What the profiler says

> Most of the time is spent in `reflow`.

```rust
fn reflow(rows: &[Row]) -> usize {
    rows.len()
}
```

| Rows | Time |
|:-----|-----:|
| 500 | 40 ms |
| 5,000 | 4 s |

- [ ] add a benchmark
- [x] reproduce the hang
";

const FONTS: [&[u8]; 4] = [
    include_bytes!("../../bourbon-ui/assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../bourbon-ui/assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../../bourbon-ui/assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../bourbon-ui/assets/fonts/Inter-Bold.ttf"),
];

struct Demo {
    editor: Entity<Editor>,
    preview: Entity<PreviewView>,
    dark: bool,
}

impl Demo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| Editor::new(NOTE, EditorTheme::light(), cx));
        let preview = cx.new(|cx| PreviewView::new(NOTE, PreviewTheme::light(), cx));
        cx.subscribe(&editor, |demo, _, event: &EditorEvent, cx| {
            if let EditorEvent::Changed(text) = event {
                demo.preview
                    .update(cx, |preview, cx| preview.set_source(text, cx));
            }
        })
        .detach();
        cx.subscribe(&preview, |demo, _, event: &PreviewEvent, cx| {
            if let PreviewEvent::ToggleTask { source_line } = event {
                demo.editor.update(cx, |editor, cx| {
                    editor.toggle_task_at_line(*source_line, cx);
                });
            }
        })
        .detach();
        let handle = editor.read(cx).focus_handle().clone();
        window.focus(&handle, cx);
        Self {
            editor,
            preview,
            dark: false,
        }
    }

    fn toggle_theme(&mut self, _: &ToggleTheme, _: &mut Window, cx: &mut Context<Self>) {
        self.dark = !self.dark;
        let (editor_theme, preview_theme) = if self.dark {
            (EditorTheme::dark(), PreviewTheme::dark())
        } else {
            (EditorTheme::light(), PreviewTheme::light())
        };
        self.editor
            .update(cx, |editor, cx| editor.set_theme(editor_theme, cx));
        self.preview
            .update(cx, |preview, cx| preview.set_theme(preview_theme, cx));
        cx.notify();
    }

    fn toggle_line_numbers(
        &mut self,
        _: &ToggleLineNumbers,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.update(cx, |editor, cx| {
            let enabled = !editor.line_numbers();
            editor.set_line_numbers(enabled, cx);
        });
    }
}

impl Focusable for Demo {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle().clone()
    }
}

impl Render for Demo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = if self.dark {
            PreviewTheme::dark().border
        } else {
            PreviewTheme::light().border
        };
        div()
            .size_full()
            .flex()
            .flex_row()
            .on_action(cx.listener(Self::toggle_theme))
            .on_action(cx.listener(Self::toggle_line_numbers))
            .child(div().flex_1().min_w_0().h_full().child(self.editor.clone()))
            .child(div().w(px(1.0)).h_full().flex_none().bg(border))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.preview.clone()),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let faces = FONTS.iter().map(|face| Cow::Borrowed(*face)).collect();
        if let Err(error) = cx.text_system().add_fonts(faces) {
            eprintln!("editor demo: cannot load fonts: {error}");
        }
        register_editor_fonts(cx);
        bind_editor_keys(cx);
        cx.bind_keys([
            KeyBinding::new("ctrl-shift-l", ToggleTheme, None),
            KeyBinding::new("ctrl-shift-n", ToggleLineNumbers, None),
        ]);
        let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some("bourbon-editor-demo".to_owned()),
                titlebar: Some(TitlebarOptions {
                    title: Some("Bourbon Editor Demo".into()),
                    ..TitlebarOptions::default()
                }),
                ..WindowOptions::default()
            },
            |window, cx| cx.new(|cx| Demo::new(window, cx)),
        )
        .expect("demo window opens");
        cx.activate(true);
    });
}
