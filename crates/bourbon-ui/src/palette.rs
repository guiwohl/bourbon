use gpui::Hsla;

pub const WHITE: u32 = 0x00ff_ffff;
pub const BLACK: u32 = 0x0000_0000;

pub const NEUTRAL_50: u32 = 0x00fa_fafa;
pub const NEUTRAL_200: u32 = 0x00e5_e5e5;
pub const NEUTRAL_200_SECTION: u32 = 0x00e6_e6e6;
pub const NEUTRAL_700: u32 = 0x0040_4040;
pub const NEUTRAL_800: u32 = 0x0026_2626;
pub const NEUTRAL_900: u32 = 0x0017_1717;
pub const NEUTRAL_950: u32 = 0x000a_0a0a;
pub const NEUTRAL_PLACEHOLDER_LIGHT: u32 = 0x00bf_bfbf;
pub const NEUTRAL_100: u32 = 0x00f5_f5f5;

pub const ZINC_300: u32 = 0x00d4_d4d8;
pub const ZINC_700: u32 = 0x003f_3f46;

pub const GRAY_50: u32 = 0x00f9_fafb;
pub const GRAY_100: u32 = 0x00f3_f4f6;
pub const GRAY_200: u32 = 0x00e5_e7eb;
pub const GRAY_300: u32 = 0x00d1_d5db;
pub const GRAY_400: u32 = 0x009c_a3af;

pub const STONE_50: u32 = 0x00fa_faf9;

pub const BLUE_200: u32 = 0x00bf_dbfe;
pub const BLUE_300: u32 = 0x0091_c3fd;
pub const BLUE_400: u32 = 0x0061_a6fa;
pub const BLUE_500: u32 = 0x003c_83f6;
pub const BLUE_600: u32 = 0x0024_63eb;
pub const BLUE_700: u32 = 0x001d_4ed8;

pub const SKY_300: u32 = 0x007e_d4fc;
pub const SKY_500: u32 = 0x000d_a2e7;
pub const SKY_600: u32 = 0x0002_84c5;
pub const SKY_800: u32 = 0x0007_5783;

pub const CYAN_400: u32 = 0x0022_d3ee;
pub const GREEN_500: u32 = 0x0021_c45d;
pub const RED_500: u32 = 0x00ef_4444;
pub const AMBER_400: u32 = 0x00fb_bf24;
pub const YELLOW_400: u32 = 0x00fa_cc15;

pub const STATUS_ACTIVE: u32 = 0x0094_a3b8;
pub const STATUS_ON_HOLD: u32 = 0x00f5_9f0a;
pub const STATUS_COMPLETED: u32 = 0x0010_b77f;
pub const STATUS_DROPPED: u32 = 0x00fb_6f84;

#[must_use]
pub fn solid(hex: u32) -> Hsla {
    gpui::rgb(hex).into()
}

#[must_use]
pub fn translucent(hex: u32, alpha: f32) -> Hsla {
    solid(hex).alpha(alpha)
}

#[must_use]
pub fn blended(base: u32, over: u32, weight: f32) -> Hsla {
    let base = gpui::rgb(base);
    let over = gpui::rgb(over);
    gpui::Rgba {
        r: base.r * (1. - weight) + over.r * weight,
        g: base.g * (1. - weight) + over.g * weight,
        b: base.b * (1. - weight) + over.b * weight,
        a: 1.,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_colours_are_opaque_and_keep_their_channels() {
        let colour = solid(0x0080_4020).to_rgb();
        assert!((colour.a - 1.).abs() < f32::EPSILON);
        assert!((colour.r - 128. / 255.).abs() < 1e-3);
        assert!((colour.g - 64. / 255.).abs() < 1e-3);
        assert!((colour.b - 32. / 255.).abs() < 1e-3);
    }

    #[test]
    fn translucent_colours_keep_the_channels_and_set_the_alpha() {
        let colour = translucent(WHITE, 0.3);
        assert!((colour.a - 0.3).abs() < f32::EPSILON);
        assert!((colour.to_rgb().r - 1.).abs() < 1e-3);
    }

    #[test]
    fn blending_half_way_averages_each_channel_and_stays_opaque() {
        let mixed = blended(BLACK, WHITE, 0.5).to_rgb();
        assert!((mixed.r - 0.5).abs() < 1e-3);
        assert!((mixed.a - 1.).abs() < f32::EPSILON);
    }
}
