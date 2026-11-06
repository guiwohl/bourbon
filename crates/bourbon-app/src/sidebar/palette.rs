use bourbon_core::TagColor;
use gpui_kit::{Hsla, rgb};

pub(super) fn dot_color(color: TagColor) -> Hsla {
    rgb(rgb_value(color)).into()
}

pub(super) fn color_label(color: TagColor) -> String {
    let stored = color.as_stored();
    let mut letters = stored.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

pub(super) fn is_current_color(current: TagColor, option: TagColor) -> bool {
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
    fn every_tag_colour_has_its_own_dot_colour() {
        let mut values: Vec<_> = TagColor::ALL.into_iter().map(rgb_value).collect();
        values.sort_unstable();
        values.dedup();
        assert_eq!(values.len(), TagColor::ALL.len());
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
