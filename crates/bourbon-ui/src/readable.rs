use gpui::{Hsla, Rgba};

pub(crate) const MINIMUM_CONTRAST: f32 = 4.5;
pub const PILL_TINT: f32 = 0.15;
const DARKENING_STEPS: u16 = 20;
const DARKENING_STEP: f32 = 0.05;

#[must_use]
pub fn pill_text_color(tag: Hsla, backdrop: Hsla, is_dark_theme: bool) -> Hsla {
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

pub(crate) fn readable_shade(tag: [f32; 3], backdrop: [f32; 3], toward: f32) -> [f32; 3] {
    let tinted = blend(backdrop, tag, PILL_TINT);
    (0..=DARKENING_STEPS)
        .map(|step| blend(tag, [toward; 3], f32::from(step) * DARKENING_STEP))
        .find(|shade| contrast(*shade, tinted) >= MINIMUM_CONTRAST)
        .unwrap_or([toward; 3])
}

pub(crate) fn blend(base: [f32; 3], other: [f32; 3], weight: f32) -> [f32; 3] {
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

pub(crate) fn contrast(first: [f32; 3], second: [f32; 3]) -> f32 {
    let (a, b) = (luminance(first), luminance(second));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAG_COLOURS: [u32; 14] = [
        0x009c_a3af,
        0x00ef_4444,
        0x00f9_7316,
        0x00ea_b308,
        0x0084_a02c,
        0x0022_c55e,
        0x0014_b8a6,
        0x003b_82f6,
        0x008b_5cf6,
        0x00a8_55f7,
        0x00ec_4899,
        0x0092_643a,
        0x006b_7280,
        0x001f_2937,
    ];

    fn channels(hex: u32) -> [f32; 3] {
        let rgba = Hsla::from(gpui::rgb(hex)).to_rgb();
        [rgba.r, rgba.g, rgba.b]
    }

    #[test]
    fn pill_text_is_readable_on_its_tint_for_every_colour_in_both_themes() {
        for (backdrop, toward) in [([0.97; 3], 0.), ([0.15; 3], 1.)] {
            for hex in TAG_COLOURS {
                let tag = channels(hex);
                let shade = readable_shade(tag, backdrop, toward);
                assert!(
                    contrast(shade, blend(backdrop, tag, PILL_TINT)) >= MINIMUM_CONTRAST,
                    "{hex:#08x}"
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
    fn a_pale_colour_is_darkened_part_of_the_way() {
        let tag = [0.8, 0.8, 0.2];
        let shade = readable_shade(tag, [0.97; 3], 0.);
        for (channel, original) in shade.into_iter().zip(tag) {
            assert!(channel > 0. && channel < original);
            let steps = channel / original;
            assert!(
                ((1. - steps) * 20.)
                    .fract()
                    .min(1. - ((1. - steps) * 20.).fract())
                    < 1e-3
            );
        }
    }

    #[test]
    fn a_colour_that_already_reads_well_is_kept() {
        let navy = [0.05, 0.1, 0.4];
        assert_eq!(readable_shade(navy, [0.97; 3], 0.), navy);
    }

    #[test]
    fn blending_mixes_each_channel_by_weight() {
        let mixed = blend([0.2, 0.4, 0.6], [0.8, 0.6, 0.2], 0.25);
        for (got, want) in mixed.into_iter().zip([0.35, 0.45, 0.5]) {
            assert!((got - want).abs() < 1e-6);
        }
    }

    #[test]
    fn luminance_follows_the_srgb_curve() {
        assert!((luminance([1.; 3]) - 1.).abs() < 1e-4);
        assert!((luminance([0.5; 3]) - 0.2140).abs() < 1e-3);
        assert!((luminance([0.02; 3]) - 0.02 / 12.92).abs() < 1e-6);
        assert!((luminance([1., 0., 0.]) - 0.2126).abs() < 1e-4);
        assert!((luminance([0., 1., 0.]) - 0.7152).abs() < 1e-4);
        assert!((luminance([0., 0., 1.]) - 0.0722).abs() < 1e-4);
    }

    #[test]
    fn opaque_colours_keep_their_channels() {
        let rgba = rgba_from([0.1, 0.2, 0.3]);
        assert_eq!((rgba.r, rgba.g, rgba.b, rgba.a), (0.1, 0.2, 0.3, 1.));
    }

    #[test]
    fn pill_text_is_the_readable_shade_made_opaque() {
        let tag = Hsla::from(gpui::rgb(0x00ea_b308));
        let backdrop = Hsla::from(gpui::rgb(0x00f7_f7f7));
        let [r, g, b] = readable_shade(channels(0x00ea_b308), [0.969; 3], 0.);
        let text = pill_text_color(tag, backdrop, false).to_rgb();
        assert!((text.a - 1.).abs() < f32::EPSILON);
        assert!((text.r - r).abs() < 0.01);
        assert!((text.g - g).abs() < 0.01 && (text.b - b).abs() < 0.01);
    }

    #[test]
    fn dark_themes_lighten_the_text_instead_of_darkening_it() {
        let tag = Hsla::from(gpui::rgb(0x001f_2937));
        let backdrop = Hsla::from(gpui::rgb(0x0017_1717));
        let text = pill_text_color(tag, backdrop, true).to_rgb();
        assert!(text.r + text.g + text.b > tag.to_rgb().r + tag.to_rgb().g + tag.to_rgb().b);
    }
}
