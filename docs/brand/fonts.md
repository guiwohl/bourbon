# Fonts

The UI font is Inter 4.1 (rsms/inter), static TTFs Regular, Medium, SemiBold and Bold in `assets/fonts/`, with its SIL Open Font License 1.1 text in `assets/fonts/Inter-OFL.txt`.

- Embedded with `include_bytes!` in `crates/bourbon-app/src/fonts.rs` and registered with `cx.text_system().add_fonts` before the window opens.
- `theme::apply` sets the gpui-component `Theme::font_family` to `Inter` for every palette, so the weights (title SemiBold, header title Medium) render the same on any machine. The mono font is unchanged.
- Reason: the system UI fonts (Liberation Sans, Noto Sans) showed no bold.
