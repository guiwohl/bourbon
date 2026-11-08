# Fonts

The UI font is Inter 4.1 (rsms/inter), static TTFs Regular, Medium, SemiBold and Bold in `crates/bourbon-ui/assets/fonts/`, with its SIL Open Font License 1.1 text in `crates/bourbon-ui/assets/fonts/Inter-OFL.txt`.

- Embedded with `include_bytes!` in `crates/bourbon-ui/src/fonts.rs` (`register_fonts`) and registered with `cx.text_system().add_fonts` before the window opens. `crates/bourbon-app/src/fonts.rs` re-exports it.
- `theme::apply` sets the gpui-component `Theme::font_family` to `Inter` for every palette, so the weights (title SemiBold, header title Medium) render the same on any machine. The mono font is unchanged.
- Reason: the system UI fonts (Liberation Sans, Noto Sans) showed no bold.

## Editor monospace

JetBrains Mono 2.304 (OFL 1.1), static Regular, Italic, Bold and BoldItalic TTFs in `crates/bourbon-editor/assets/fonts/` with `JetBrainsMono-OFL.txt`. `bourbon_editor::register_editor_fonts(cx)` registers them; `EditorTheme` and `PreviewTheme` use the family `JetBrains Mono`. The system "monospace" names did not resolve on Linux in GPUI.
