use std::borrow::Cow;

use gpui::App;

pub const UI_FAMILY: &str = "Inter";

const FACES: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Inter-Bold.ttf"),
];

pub fn register_fonts(cx: &mut App) {
    let faces = FACES.iter().map(|face| Cow::Borrowed(*face)).collect();
    if let Err(error) = cx.text_system().add_fonts(faces) {
        eprintln!("bourbon: cannot load the bundled fonts: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_face_is_a_truetype_font() {
        assert_eq!(FACES.len(), 4);
        for face in FACES {
            assert_eq!(&face[..4], &[0, 1, 0, 0]);
        }
    }

    #[test]
    fn the_ui_family_is_the_token_family() {
        assert_eq!(
            UI_FAMILY,
            crate::tokens::Fonts::bourbon().ui_family.as_ref()
        );
    }
}
