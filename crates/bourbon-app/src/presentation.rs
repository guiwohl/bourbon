use bourbon_core::{NoteStatus, TagColor};
use bourbon_ui::{Colors, IconName};
use gpui::{Hsla, rgb};
use gpui_kit::component::ColorName;

pub(crate) const UNTITLED: &str = "Untitled";

pub(crate) fn display_title(title: &str) -> &str {
    if title.is_empty() { UNTITLED } else { title }
}

pub(crate) fn status_label(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::None => "None",
        NoteStatus::Active => "Active",
        NoteStatus::OnHold => "On Hold",
        NoteStatus::Completed => "Completed",
        NoteStatus::Dropped => "Dropped",
    }
}

pub(crate) fn status_icon(status: NoteStatus, colors: &Colors) -> Option<(IconName, Hsla)> {
    match status {
        NoteStatus::None => None,
        NoteStatus::Active => Some((IconName::CirclePlay, colors.status_active)),
        NoteStatus::OnHold => Some((IconName::CirclePause, colors.status_on_hold)),
        NoteStatus::Completed => Some((IconName::CircleCheck, colors.status_completed)),
        NoteStatus::Dropped => Some((IconName::CircleX, colors.status_dropped)),
    }
}

pub(crate) fn chip_color(color: TagColor) -> ColorName {
    match color {
        TagColor::Default => ColorName::Gray,
        TagColor::Red => ColorName::Red,
        TagColor::Orange => ColorName::Orange,
        TagColor::Yellow => ColorName::Yellow,
        TagColor::Olive => ColorName::Lime,
        TagColor::Green => ColorName::Green,
        TagColor::Teal => ColorName::Teal,
        TagColor::Blue => ColorName::Blue,
        TagColor::Violet => ColorName::Violet,
        TagColor::Purple => ColorName::Purple,
        TagColor::Pink => ColorName::Pink,
        TagColor::Brown => ColorName::Amber,
        TagColor::Grey => ColorName::Neutral,
        TagColor::Black => ColorName::Black,
    }
}

pub(crate) fn dot_color(color: TagColor) -> Hsla {
    rgb(rgb_value(color)).into()
}

pub(crate) fn color_label(color: TagColor) -> String {
    let stored = color.as_stored();
    let mut letters = stored.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

pub(crate) fn is_current_color(current: TagColor, option: TagColor) -> bool {
    current == option
}

fn rgb_value(color: TagColor) -> u32 {
    match color {
        TagColor::Default => 0x009c_a3af,
        TagColor::Red => 0x00ef_4444,
        TagColor::Orange => 0x00f9_7316,
        TagColor::Yellow => 0x00ea_b308,
        TagColor::Olive => 0x0084_a02c,
        TagColor::Green => 0x0022_c55e,
        TagColor::Teal => 0x0014_b8a6,
        TagColor::Blue => 0x003b_82f6,
        TagColor::Violet => 0x008b_5cf6,
        TagColor::Purple => 0x00a8_55f7,
        TagColor::Pink => 0x00ec_4899,
        TagColor::Brown => 0x0092_643a,
        TagColor::Grey => 0x006b_7280,
        TagColor::Black => 0x001f_2937,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_notes_with_a_status_get_a_status_icon_in_the_status_colour() {
        let colors = bourbon_ui::Tokens::light().colors;
        assert!(status_icon(NoteStatus::None, &colors).is_none());
        assert_eq!(
            status_icon(NoteStatus::Active, &colors),
            Some((IconName::CirclePlay, colors.status_active))
        );
        assert_eq!(
            status_icon(NoteStatus::Completed, &colors),
            Some((IconName::CircleCheck, colors.status_completed))
        );
        let mut icons: Vec<_> = NoteStatus::ALL
            .into_iter()
            .filter_map(|status| status_icon(status, &colors))
            .map(|(icon, _)| icon.file_name())
            .collect();
        icons.sort_unstable();
        icons.dedup();
        assert_eq!(icons.len(), 4);
    }

    #[test]
    fn a_blank_title_is_shown_as_untitled() {
        assert_eq!(display_title(""), "Untitled");
        assert_eq!(display_title("Plan"), "Plan");
    }

    #[test]
    fn every_tag_colour_has_its_own_dot_colour() {
        let mut values: Vec<_> = TagColor::ALL.into_iter().map(rgb_value).collect();
        values.sort_unstable();
        values.dedup();
        assert_eq!(values.len(), TagColor::ALL.len());
    }

    #[test]
    fn every_tag_colour_has_its_own_chip_colour() {
        let mut names: Vec<_> = TagColor::ALL
            .into_iter()
            .map(|color| format!("{:?}", chip_color(color)))
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), TagColor::ALL.len());
    }

    #[test]
    fn every_status_has_a_distinct_label() {
        let labels: Vec<_> = NoteStatus::ALL.into_iter().map(status_label).collect();
        assert_eq!(
            labels,
            ["None", "Active", "On Hold", "Completed", "Dropped"]
        );
    }

    #[test]
    fn only_the_current_colour_is_checked() {
        assert!(is_current_color(TagColor::Red, TagColor::Red));
        assert!(!is_current_color(TagColor::Red, TagColor::Blue));
    }

    #[test]
    fn colour_labels_are_capitalised_names() {
        assert_eq!(color_label(TagColor::Default), "Default");
        assert_eq!(color_label(TagColor::Violet), "Violet");
    }

    #[test]
    fn the_dot_colour_is_the_palette_value() {
        assert_eq!(dot_color(TagColor::Red), Hsla::from(rgb(0x00ef_4444)));
    }
}
