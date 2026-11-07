use bourbon_core::NoteStatus;
use gpui_kit::assets::{IconName, icon_assets};
use gpui_kit::component::{Icon, Sizable as _, Size};
use gpui_kit::{AssetSource as _, Hsla, Styled as _, px, rgb};

icon_assets!(
    Glyphs,
    [
        Pin,
        Funnel,
        SquarePen,
        Trash,
        Notebook,
        Inbox,
        FileText,
        Copy,
        Settings,
        Search,
        Plus,
        ChevronUp,
        ChevronDown,
        ChevronRight,
        PanelLeft,
        CirclePlay,
        CirclePause,
        CircleCheck,
        CircleX,
        ListChecks,
        Check,
    ]
);

const ICON_SIZE: f32 = 16.;

const STATUS_BLUE: u32 = 0x003d_7be0;
const STATUS_ORANGE: u32 = 0x00f0_a020;
const STATUS_GREEN: u32 = 0x002f_b36a;
const STATUS_RED: u32 = 0x00e5_534b;

pub(crate) fn glyph(name: IconName) -> Icon {
    Glyphs
        .load(&name.path())
        .ok()
        .flatten()
        .map_or_else(Icon::empty, |bytes| Icon::empty().data(&bytes))
        .with_size(Size::Size(px(ICON_SIZE)))
}

pub(crate) fn status_glyph(status: NoteStatus) -> Option<Icon> {
    let (name, color) = status_style(status)?;
    Some(glyph(name).text_color(Hsla::from(rgb(color))))
}

fn status_style(status: NoteStatus) -> Option<(IconName, u32)> {
    match status {
        NoteStatus::None => None,
        NoteStatus::Active => Some((IconName::CirclePlay, STATUS_BLUE)),
        NoteStatus::OnHold => Some((IconName::CirclePause, STATUS_ORANGE)),
        NoteStatus::Completed => Some((IconName::CircleCheck, STATUS_GREEN)),
        NoteStatus::Dropped => Some((IconName::CircleX, STATUS_RED)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_notes_with_a_status_get_a_status_icon() {
        assert!(status_style(NoteStatus::None).is_none());
        assert!(
            NoteStatus::ALL
                .into_iter()
                .filter(|status| *status != NoteStatus::None)
                .all(|status| status_style(status).is_some())
        );
    }

    #[test]
    fn a_status_glyph_exists_only_for_notes_with_a_status() {
        assert!(status_glyph(NoteStatus::None).is_none());
        assert!(status_glyph(NoteStatus::Active).is_some());
    }

    #[test]
    fn each_status_has_its_own_colour() {
        let mut colors: Vec<_> = NoteStatus::ALL
            .into_iter()
            .filter_map(status_style)
            .map(|(_, color)| color)
            .collect();
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), 4);
    }

    #[test]
    fn every_glyph_is_embedded() {
        assert!(Glyphs.load(&IconName::Funnel.path()).unwrap().is_some());
        assert!(Glyphs.load(&IconName::SquarePen.path()).unwrap().is_some());
    }
}
