use std::borrow::Cow;

use gpui::{App, FontFeatures};

pub const MONO_FAMILY: &str = "JetBrains Mono";

const FACES: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf"),
];

pub fn without_ligatures() -> FontFeatures {
    FontFeatures(std::sync::Arc::new(vec![
        ("calt".to_owned(), 0),
        ("liga".to_owned(), 0),
    ]))
}

pub fn register_editor_fonts(cx: &App) -> usize {
    let faces = FACES.iter().map(|face| Cow::Borrowed(*face)).collect();
    match cx.text_system().add_fonts(faces) {
        Ok(()) => FACES.len(),
        Err(error) => {
            eprintln!("bourbon-editor: cannot load the monospace font: {error}");
            0
        }
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

    #[gpui::test]
    fn registering_reports_every_face(cx: &mut gpui::TestAppContext) {
        assert_eq!(cx.update(|cx| register_editor_fonts(cx)), 4);
    }

    #[test]
    fn ligatures_are_switched_off() {
        let features = without_ligatures();
        assert_eq!(
            features.tag_value_list(),
            [("calt".to_owned(), 0), ("liga".to_owned(), 0)]
        );
    }

    #[test]
    fn themes_use_the_bundled_family() {
        assert_eq!(crate::EditorTheme::light().font_family, MONO_FAMILY);
        assert_eq!(crate::PreviewTheme::light().mono_family, MONO_FAMILY);
    }
}
