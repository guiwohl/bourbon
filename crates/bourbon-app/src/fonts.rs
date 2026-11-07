use std::borrow::Cow;

use gpui_kit::App;

pub(crate) const UI_FAMILY: &str = "Inter";

const FACES: [&[u8]; 4] = [
    include_bytes!("../../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/Inter-Bold.ttf"),
];

pub(crate) fn register(cx: &mut App) {
    let faces = FACES.iter().map(|face| Cow::Borrowed(*face)).collect();
    if let Err(error) = cx.text_system().add_fonts(faces) {
        eprintln!("bourbon: cannot load the bundled fonts: {error}");
    }
}
