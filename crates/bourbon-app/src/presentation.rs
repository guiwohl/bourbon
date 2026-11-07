use bourbon_core::{NoteStatus, TagColor};
use gpui_kit::component::ColorName;
use gpui_kit::{Hsla, Rgba, rgb};

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

const MINIMUM_CONTRAST: f32 = 4.5;
const PILL_TINT: f32 = 0.15;
const DARKENING_STEPS: u16 = 20;

pub(crate) fn pill_text_color(tag: Hsla, backdrop: Hsla, is_dark_theme: bool) -> Hsla {
    let tag = tag.to_rgb();
    let backdrop = backdrop.to_rgb();
    let toward = if is_dark_theme { 1. } else { 0. };
    let tag = [tag.r, tag.g, tag.b];
    let backdrop = [backdrop.r, backdrop.g, backdrop.b];
    let readable = readable_shade(tag, backdrop, toward);
    rgba_from(readable).into()
}

fn rgba_from(channels: [f32; 3]) -> Rgba {
    Rgba {
        r: channels[0],
        g: channels[1],
        b: channels[2],
        a: 1.,
    }
}

fn readable_shade(tag: [f32; 3], backdrop: [f32; 3], toward: f32) -> [f32; 3] {
    let tinted = blend(backdrop, tag, PILL_TINT);
    (0..=DARKENING_STEPS)
        .map(|step| {
            blend(
                tag,
                [toward; 3],
                f32::from(step) / f32::from(DARKENING_STEPS),
            )
        })
        .find(|shade| contrast(*shade, tinted) >= MINIMUM_CONTRAST)
        .unwrap_or([toward; 3])
}

fn blend(base: [f32; 3], other: [f32; 3], weight: f32) -> [f32; 3] {
    [0, 1, 2].map(|channel| base[channel] * (1. - weight) + other[channel] * weight)
}

fn luminance(color: [f32; 3]) -> f32 {
    let linear = color.map(|channel| {
        if channel <= 0.03928 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]
}

fn contrast(first: [f32; 3], second: [f32; 3]) -> f32 {
    let (a, b) = (luminance(first), luminance(second));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
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

    fn channels(color: TagColor) -> [f32; 3] {
        let rgba = dot_color(color).to_rgb();
        [rgba.r, rgba.g, rgba.b]
    }

    #[test]
    fn pill_text_is_readable_on_its_tint_for_every_colour_in_both_themes() {
        for (backdrop, toward) in [([0.97; 3], 0.), ([0.15; 3], 1.)] {
            for color in TagColor::ALL {
                let tag = channels(color);
                let shade = readable_shade(tag, backdrop, toward);
                assert!(
                    contrast(shade, blend(backdrop, tag, PILL_TINT)) >= MINIMUM_CONTRAST,
                    "{color:?}"
                );
            }
        }
    }

    #[test]
    fn contrast_between_white_and_black_is_twenty_one() {
        assert!((contrast([1.; 3], [0.; 3]) - 21.).abs() < 1e-3);
        assert!((contrast([0.; 3], [1.; 3]) - 21.).abs() < 1e-3);
        assert!((contrast([0.5; 3], [0.5; 3]) - 1.).abs() < 1e-6);
    }

    #[test]
    fn a_colour_that_already_reads_well_is_kept() {
        let navy = [0.05, 0.1, 0.4];
        assert_eq!(readable_shade(navy, [0.97; 3], 0.), navy);
    }

    #[test]
    fn the_dot_colour_is_the_palette_value() {
        assert_eq!(dot_color(TagColor::Red), Hsla::from(rgb(0x00ef_4444)));
    }
}
