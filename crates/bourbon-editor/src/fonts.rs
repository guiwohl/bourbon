use std::borrow::Cow;

use gpui::App;

pub const MONO_FAMILY: &str = "JetBrains Mono";

const FACES: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf"),
];

pub fn register_editor_fonts(cx: &App) {
    let faces = FACES.iter().map(|face| Cow::Borrowed(*face)).collect();
    if let Err(error) = cx.text_system().add_fonts(faces) {
        eprintln!("bourbon-editor: cannot load the monospace font: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_face_is_a_font_file() {
        assert_eq!(FACES.len(), 4);
        assert!(FACES.iter().all(|face| face.len() > 100_000));
    }

    #[test]
    fn themes_use_the_bundled_family() {
        assert_eq!(crate::EditorTheme::light().font_family, MONO_FAMILY);
        assert_eq!(crate::PreviewTheme::light().mono_family, MONO_FAMILY);
    }
}
