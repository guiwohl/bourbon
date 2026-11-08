use std::time::Duration;

use gpui::{BoxShadow, FontWeight, Hsla, Pixels, SharedString, point, px};

use crate::palette::{
    AMBER_400, BLACK, BLUE_200, BLUE_300, BLUE_400, BLUE_500, BLUE_600, BLUE_700, CYAN_400,
    GRAY_50, GRAY_100, GRAY_200, GRAY_300, GRAY_400, GREEN_500, NEUTRAL_50, NEUTRAL_100,
    NEUTRAL_200, NEUTRAL_200_SECTION, NEUTRAL_700, NEUTRAL_800, NEUTRAL_900, NEUTRAL_950,
    NEUTRAL_PLACEHOLDER_LIGHT, RED_500, SKY_300, SKY_500, SKY_600, SKY_800, STATUS_ACTIVE,
    STATUS_COMPLETED, STATUS_DROPPED, STATUS_ON_HOLD, STONE_50, WHITE, YELLOW_400, ZINC_300,
    ZINC_700, blended, solid, translucent,
};
use crate::splitter::WidthLimits;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub struct Tokens {
    pub scale: Scale,
    pub fonts: Fonts,
    pub colors: Colors,
    pub shadows: Shadows,
}

#[derive(Clone, Copy, Debug)]
pub struct Scale {
    pub mini: Pixels,
    pub tiny: Pixels,
    pub small: Pixels,
    pub medium: Pixels,
    pub large: Pixels,
    pub big: Pixels,
    pub huge: Pixels,
    pub massive: Pixels,
    pub line_height: Pixels,
    pub weight_normal: FontWeight,
    pub weight_medium: FontWeight,
    pub weight_semibold: FontWeight,
    pub weight_bold: FontWeight,
    pub radius_default: Pixels,
    pub radius_control: Pixels,
    pub radius_large: Pixels,
    pub radius_tag: Pixels,
    pub radius_pill: Pixels,
    pub disabled_opacity: f32,
    pub scrollbar_width: Pixels,
    pub scrollbar_hover_width: Pixels,
    pub focus_ring_width: Pixels,
    pub focus_ring_offset: Pixels,
    pub transition: Duration,
    pub transition_panel: Duration,
    pub transition_chevron: Duration,
    pub transition_modal: Duration,
    pub transition_header: Duration,
    pub sidebar_width: WidthLimits,
    pub note_list_width: WidthLimits,
    pub header_height: Pixels,
    pub search_bar_height: Pixels,
    pub sidebar_row_height: Pixels,
    pub sidebar_footer_height: Pixels,
    pub sidebar_row_indent: Pixels,
    pub sidebar_row_step: Pixels,
    pub sidebar_row_trailing_padding: Pixels,
    pub sidebar_accent_bar: Pixels,
    pub row_gap: Pixels,
    pub icon_slot: Pixels,
    pub tag_dot_slot: Pixels,
    pub icon_button_icon: Pixels,
    pub icon_button_size: Pixels,
    pub accessory_button_size: Pixels,
    pub segment_width: Pixels,
    pub segment_height: Pixels,
    pub menu_item_padding_vertical: Pixels,
    pub menu_item_padding_horizontal: Pixels,
    pub menu_min_width: Pixels,
    pub tooltip_padding_vertical: Pixels,
    pub tooltip_padding_horizontal: Pixels,
    pub splitter_handle: Pixels,
    pub border: Pixels,
}

#[derive(Clone, Debug)]
pub struct Fonts {
    pub ui_family: SharedString,
    pub mono_family: SharedString,
    pub mono_stack: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub struct Colors {
    pub page_background: Hsla,
    pub text: Hsla,
    pub muted_text: Hsla,
    pub light_text: Hsla,
    pub dark_text: Hsla,
    pub unselected_text: Hsla,
    pub hovered_text: Hsla,
    pub pressed_text: Hsla,
    pub selected_text: Hsla,
    pub disabled_text: Hsla,
    pub inverted_text: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_active: Hsla,
    pub link: Hsla,
    pub highlight_background: Hsla,
    pub border: Hsla,
    pub strong_border: Hsla,
    pub internal_border: Hsla,
    pub selected_border: Hsla,
    pub strong_selected_border: Hsla,
    pub solid_border: Hsla,
    pub solid_internal_border: Hsla,
    pub subtle_transparent_black: Hsla,
    pub transparent_black: Hsla,
    pub strong_transparent_black: Hsla,
    pub very_strong_transparent_black: Hsla,
    pub subtle_transparent_white: Hsla,
    pub transparent_white: Hsla,
    pub strong_transparent_white: Hsla,
    pub focus_outline: Hsla,
    pub focused_form_border: Hsla,
    pub input_background: Hsla,
    pub input_text: Hsla,
    pub input_placeholder: Hsla,
    pub positive: Hsla,
    pub negative: Hsla,
    pub info: Hsla,
    pub warning: Hsla,
    pub sidebar_background: Hsla,
    pub sidebar_border: Hsla,
    pub note_list_background: Hsla,
    pub note_list_border: Hsla,
    pub note_list_item_separator: Hsla,
    pub editor_background: Hsla,
    pub section_header_background: Hsla,
    pub floating_actions_background: Hsla,
    pub menu_background: Hsla,
    pub modal_actions_background: Hsla,
    pub backdrop: Hsla,
    pub selection_inactive: Hsla,
    pub selection_active_sidebar: Hsla,
    pub selection_active_note_list: Hsla,
    pub sidebar_item_text: Hsla,
    pub sidebar_item_active_text: Hsla,
    pub sidebar_section_text: Hsla,
    pub sidebar_section_separator: Hsla,
    pub sidebar_action_icon: Hsla,
    pub sidebar_action_icon_hover: Hsla,
    pub note_list_date: Hsla,
    pub button_background: Hsla,
    pub button_hover_background: Hsla,
    pub button_down_background: Hsla,
    pub button_active_background: Hsla,
    pub button_text: Hsla,
    pub button_ring: Hsla,
    pub selection_list_hover: Hsla,
    pub selection_list_active: Hsla,
    pub menu_hover: Hsla,
    pub menu_active: Hsla,
    pub scrollbar_track: Hsla,
    pub scrollbar_thumb: Hsla,
    pub kbd_background: Hsla,
    pub kbd_border: Hsla,
    pub mark_background: Hsla,
    pub mark_border: Hsla,
    pub status_active: Hsla,
    pub status_on_hold: Hsla,
    pub status_completed: Hsla,
    pub status_dropped: Hsla,
}

#[derive(Clone, Debug)]
pub struct Shadows {
    pub floating: Vec<BoxShadow>,
    pub raised: Vec<BoxShadow>,
    pub modal: Vec<BoxShadow>,
    pub palette: Vec<BoxShadow>,
}

const MODAL_SHADOW_LAYERS: [(f32, f32, f32); 6] = [
    (1., 2.8, 0.067),
    (2., 6.7, 0.096),
    (4., 12.5, 0.12),
    (8., 22.3, 0.144),
    (16., 41.8, 0.173),
    (40., 100., 0.24),
];

impl Scale {
    #[must_use]
    pub fn inkdrop() -> Self {
        Self {
            mini: px(11.),
            tiny: px(12.),
            small: px(13.),
            medium: px(14.),
            large: px(16.),
            big: px(18.),
            huge: px(20.),
            massive: px(24.),
            line_height: px(20.),
            weight_normal: FontWeight::NORMAL,
            weight_medium: FontWeight::MEDIUM,
            weight_semibold: FontWeight::SEMIBOLD,
            weight_bold: FontWeight::BOLD,
            radius_default: px(7.),
            radius_control: px(12.),
            radius_large: px(18.),
            radius_tag: px(12.),
            radius_pill: px(500.),
            disabled_opacity: 0.45,
            scrollbar_width: px(6.),
            scrollbar_hover_width: px(14.),
            focus_ring_width: px(2.),
            focus_ring_offset: px(5.6),
            transition: Duration::from_millis(100),
            transition_panel: Duration::from_millis(400),
            transition_chevron: Duration::from_millis(200),
            transition_modal: Duration::from_millis(200),
            transition_header: Duration::from_millis(1200),
            sidebar_width: WidthLimits {
                minimum: px(170.),
                default: px(200.),
                maximum: px(500.),
            },
            note_list_width: WidthLimits {
                minimum: px(220.),
                default: px(260.),
                maximum: px(500.),
            },
            header_height: px(40.),
            search_bar_height: px(34.),
            sidebar_row_height: px(30.),
            sidebar_footer_height: px(40.),
            sidebar_row_indent: px(14.),
            sidebar_row_step: px(12.),
            sidebar_row_trailing_padding: px(6.),
            sidebar_accent_bar: px(5.),
            row_gap: px(4.),
            icon_slot: px(16.),
            tag_dot_slot: px(20.),
            icon_button_icon: px(18.),
            icon_button_size: px(30.),
            accessory_button_size: px(22.),
            segment_width: px(44.),
            segment_height: px(28.),
            menu_item_padding_vertical: px(11.),
            menu_item_padding_horizontal: px(16.),
            menu_min_width: px(180.),
            tooltip_padding_vertical: px(11.7),
            tooltip_padding_horizontal: px(14.),
            splitter_handle: px(10.),
            border: px(1.),
        }
    }
}

impl Fonts {
    #[must_use]
    pub fn bourbon() -> Self {
        Self {
            ui_family: SharedString::new_static("Inter"),
            mono_family: SharedString::new_static("Liberation Mono"),
            mono_stack: &[
                "SFMono-Regular",
                "Consolas",
                "Liberation Mono",
                "Menlo",
                "Courier",
                "monospace",
            ],
        }
    }
}

fn black_shadow(y: f32, blur: f32, spread: f32, alpha: f32) -> BoxShadow {
    BoxShadow {
        color: translucent(BLACK, alpha),
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

impl Shadows {
    fn with_floating_alphas(first: f32, second: f32) -> Self {
        Self {
            floating: vec![
                black_shadow(2., 4., 0., first),
                black_shadow(2., 10., 0., second),
            ],
            raised: vec![
                black_shadow(1.5, 3., -2., 0.4),
                black_shadow(4., 6., 0., 0.2),
                black_shadow(8., 12., -1., 0.1),
            ],
            modal: MODAL_SHADOW_LAYERS
                .iter()
                .map(|&(y, blur, alpha)| black_shadow(y, blur, 0., alpha))
                .collect(),
            palette: vec![black_shadow(10., 30., 0., 0.3)],
        }
    }
}

impl Tokens {
    #[must_use]
    pub fn for_appearance(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self::light(),
            Appearance::Dark => Self::dark(),
        }
    }

    #[must_use]
    pub fn light() -> Self {
        Self {
            scale: Scale::inkdrop(),
            fonts: Fonts::bourbon(),
            colors: Colors::light(),
            shadows: Shadows::with_floating_alphas(0.12, 0.15),
        }
    }

    #[must_use]
    pub fn dark() -> Self {
        Self {
            scale: Scale::inkdrop(),
            fonts: Fonts::bourbon(),
            colors: Colors::dark(),
            shadows: Shadows::with_floating_alphas(0.2, 0.3),
        }
    }
}

impl Colors {
    #[must_use]
    pub fn light() -> Self {
        let text = translucent(BLACK, 0.87);
        let primary = solid(BLUE_500);
        let border = translucent(NEUTRAL_900, 0.15);
        Self {
            page_background: solid(WHITE),
            text,
            muted_text: translucent(BLACK, 0.6),
            light_text: translucent(BLACK, 0.4),
            dark_text: translucent(BLACK, 0.85),
            unselected_text: translucent(BLACK, 0.4),
            hovered_text: translucent(BLACK, 0.8),
            pressed_text: translucent(BLACK, 0.9),
            selected_text: translucent(BLACK, 0.95),
            disabled_text: translucent(NEUTRAL_800, 0.3),
            inverted_text: translucent(WHITE, 0.9),
            primary,
            primary_hover: solid(BLUE_600),
            primary_active: solid(BLUE_700),
            link: solid(BLUE_500),
            highlight_background: solid(BLUE_200),
            border,
            strong_border: translucent(NEUTRAL_900, 0.22),
            internal_border: translucent(NEUTRAL_900, 0.10),
            selected_border: translucent(NEUTRAL_900, 0.35),
            strong_selected_border: translucent(NEUTRAL_900, 0.50),
            solid_border: solid(ZINC_300),
            solid_internal_border: solid(NEUTRAL_50),
            subtle_transparent_black: translucent(BLACK, 0.03),
            transparent_black: translucent(BLACK, 0.05),
            strong_transparent_black: translucent(BLACK, 0.10),
            very_strong_transparent_black: translucent(BLACK, 0.15),
            subtle_transparent_white: translucent(WHITE, 0.02),
            transparent_white: translucent(WHITE, 0.08),
            strong_transparent_white: translucent(WHITE, 0.15),
            focus_outline: translucent(SKY_500, 0.4),
            focused_form_border: translucent(SKY_600, 0.5),
            input_background: solid(WHITE),
            input_text: text,
            input_placeholder: translucent(NEUTRAL_PLACEHOLDER_LIGHT, 0.87),
            positive: solid(GREEN_500),
            negative: solid(RED_500),
            info: solid(SKY_500),
            warning: solid(AMBER_400),
            sidebar_background: solid(GRAY_100),
            sidebar_border: border,
            note_list_background: solid(STONE_50),
            note_list_border: solid(GRAY_200),
            note_list_item_separator: solid(GRAY_200),
            editor_background: solid(WHITE),
            section_header_background: solid(NEUTRAL_200_SECTION),
            floating_actions_background: translucent(WHITE, 0.6),
            menu_background: solid(WHITE),
            modal_actions_background: solid(GRAY_50),
            backdrop: translucent(BLACK, 0.3),
            selection_inactive: translucent(SKY_800, 0.2),
            selection_active_sidebar: primary,
            selection_active_note_list: primary,
            sidebar_item_text: translucent(BLACK, 0.85),
            sidebar_item_active_text: translucent(BLACK, 0.85),
            sidebar_section_text: translucent(BLACK, 0.4),
            sidebar_section_separator: translucent(NEUTRAL_900, 0.10),
            sidebar_action_icon: translucent(BLACK, 0.4),
            sidebar_action_icon_hover: translucent(BLACK, 0.8),
            note_list_date: primary,
            button_background: solid(GRAY_200),
            button_hover_background: solid(GRAY_300),
            button_down_background: solid(GRAY_400),
            button_active_background: solid(GRAY_400),
            button_text: translucent(BLACK, 0.6),
            button_ring: translucent(BLACK, 0.10),
            selection_list_hover: translucent(BLACK, 0.03),
            selection_list_active: translucent(BLACK, 0.05),
            menu_hover: translucent(BLACK, 0.03),
            menu_active: translucent(BLACK, 0.05),
            scrollbar_track: translucent(BLACK, 0.05),
            scrollbar_thumb: translucent(BLACK, 0.2),
            kbd_background: solid(NEUTRAL_50),
            kbd_border: border,
            mark_background: translucent(YELLOW_400, 0.2),
            mark_border: translucent(YELLOW_400, 0.4),
            status_active: solid(STATUS_ACTIVE),
            status_on_hold: solid(STATUS_ON_HOLD),
            status_completed: solid(STATUS_COMPLETED),
            status_dropped: solid(STATUS_DROPPED),
        }
    }

    #[must_use]
    pub fn dark() -> Self {
        let text = translucent(NEUTRAL_50, 0.87);
        let primary = translucent(BLUE_400, 0.8);
        let border = translucent(NEUTRAL_100, 0.10);
        Self {
            page_background: solid(NEUTRAL_950),
            text,
            muted_text: translucent(NEUTRAL_50, 0.6),
            light_text: translucent(NEUTRAL_50, 0.85),
            dark_text: translucent(NEUTRAL_50, 0.4),
            unselected_text: translucent(NEUTRAL_50, 0.4),
            hovered_text: translucent(NEUTRAL_50, 0.8),
            pressed_text: translucent(NEUTRAL_50, 0.9),
            selected_text: translucent(NEUTRAL_50, 0.95),
            disabled_text: translucent(NEUTRAL_50, 0.3),
            inverted_text: translucent(NEUTRAL_950, 0.9),
            primary,
            primary_hover: solid(BLUE_600),
            primary_active: solid(BLUE_700),
            link: solid(BLUE_500),
            highlight_background: solid(SKY_600),
            border,
            strong_border: translucent(NEUTRAL_100, 0.22),
            internal_border: translucent(NEUTRAL_100, 0.10),
            selected_border: translucent(NEUTRAL_100, 0.35),
            strong_selected_border: translucent(NEUTRAL_100, 0.50),
            solid_border: solid(ZINC_700),
            solid_internal_border: solid(NEUTRAL_800),
            subtle_transparent_black: translucent(BLACK, 0.06),
            transparent_black: translucent(BLACK, 0.10),
            strong_transparent_black: translucent(BLACK, 0.20),
            very_strong_transparent_black: translucent(BLACK, 0.30),
            subtle_transparent_white: translucent(WHITE, 0.02),
            transparent_white: translucent(WHITE, 0.08),
            strong_transparent_white: translucent(WHITE, 0.15),
            focus_outline: translucent(SKY_300, 0.3),
            focused_form_border: translucent(SKY_300, 0.6),
            input_background: solid(NEUTRAL_900),
            input_text: translucent(NEUTRAL_50, 0.85),
            input_placeholder: translucent(NEUTRAL_200, 0.25),
            positive: solid(GREEN_500),
            negative: solid(RED_500),
            info: solid(CYAN_400),
            warning: solid(AMBER_400),
            sidebar_background: solid(NEUTRAL_900),
            sidebar_border: solid(NEUTRAL_800),
            note_list_background: blended(NEUTRAL_950, NEUTRAL_900, 0.5),
            note_list_border: translucent(NEUTRAL_800, 0.75),
            note_list_item_separator: translucent(NEUTRAL_800, 0.75),
            editor_background: solid(NEUTRAL_950),
            section_header_background: solid(NEUTRAL_900),
            floating_actions_background: translucent(NEUTRAL_800, 0.6),
            menu_background: solid(NEUTRAL_900),
            modal_actions_background: solid(NEUTRAL_800),
            backdrop: translucent(BLACK, 0.3),
            selection_inactive: translucent(BLUE_300, 0.2),
            selection_active_sidebar: translucent(BLUE_300, 0.3),
            selection_active_note_list: translucent(BLUE_300, 0.4),
            sidebar_item_text: translucent(NEUTRAL_50, 0.6),
            sidebar_item_active_text: text,
            sidebar_section_text: translucent(NEUTRAL_50, 0.4),
            sidebar_section_separator: translucent(WHITE, 0.10),
            sidebar_action_icon: translucent(NEUTRAL_50, 0.3),
            sidebar_action_icon_hover: translucent(NEUTRAL_50, 0.85),
            note_list_date: primary,
            button_background: translucent(NEUTRAL_800, 0.7),
            button_hover_background: translucent(NEUTRAL_700, 0.7),
            button_down_background: translucent(NEUTRAL_800, 0.8),
            button_active_background: translucent(NEUTRAL_700, 0.7),
            button_text: translucent(NEUTRAL_50, 0.6),
            button_ring: translucent(WHITE, 0.15),
            selection_list_hover: translucent(WHITE, 0.02),
            selection_list_active: translucent(WHITE, 0.08),
            menu_hover: translucent(WHITE, 0.02),
            menu_active: translucent(WHITE, 0.08),
            scrollbar_track: translucent(WHITE, 0.02),
            scrollbar_thumb: translucent(WHITE, 0.2),
            kbd_background: translucent(WHITE, 0.02),
            kbd_border: border,
            mark_background: translucent(YELLOW_400, 0.2),
            mark_border: translucent(YELLOW_400, 0.4),
            status_active: solid(STATUS_ACTIVE),
            status_on_hold: solid(STATUS_ON_HOLD),
            status_completed: solid(STATUS_COMPLETED),
            status_dropped: solid(STATUS_DROPPED),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque(color: Hsla) -> bool {
        (color.a - 1.).abs() < f32::EPSILON
    }

    #[test]
    fn light_and_dark_differ_where_the_research_says_they_do() {
        let (light, dark) = (Colors::light(), Colors::dark());
        assert_ne!(light.page_background, dark.page_background);
        assert_ne!(light.text, dark.text);
        assert_ne!(light.sidebar_background, dark.sidebar_background);
        assert_ne!(light.primary, dark.primary);
        assert_ne!(light.selection_inactive, dark.selection_inactive);
        assert_ne!(light.border, dark.border);
        assert_ne!(light.scrollbar_thumb, dark.scrollbar_thumb);
    }

    #[test]
    fn surfaces_are_opaque_in_both_appearances() {
        for colors in [Colors::light(), Colors::dark()] {
            for surface in [
                colors.page_background,
                colors.editor_background,
                colors.sidebar_background,
                colors.note_list_background,
                colors.menu_background,
                colors.input_background,
                colors.section_header_background,
                colors.solid_border,
                colors.solid_internal_border,
            ] {
                assert!(opaque(surface));
            }
        }
    }

    #[test]
    fn text_and_border_ramps_stay_translucent_so_they_work_on_tinted_surfaces() {
        for colors in [Colors::light(), Colors::dark()] {
            for ramp in [
                colors.text,
                colors.muted_text,
                colors.border,
                colors.internal_border,
                colors.selection_inactive,
            ] {
                assert!(!opaque(ramp));
            }
        }
    }

    #[test]
    fn the_text_alpha_ramp_matches_the_research() {
        let light = Colors::light();
        assert!((light.text.a - 0.87).abs() < 1e-6);
        assert!((light.muted_text.a - 0.60).abs() < 1e-6);
        assert!((light.light_text.a - 0.40).abs() < 1e-6);
        let dark = Colors::dark();
        assert!((dark.text.a - 0.87).abs() < 1e-6);
        assert!((dark.light_text.a - 0.85).abs() < 1e-6);
        assert!((dark.dark_text.a - 0.40).abs() < 1e-6);
    }

    #[test]
    fn the_selection_pair_follows_the_pane_focus_tokens() {
        let light = Colors::light();
        assert_eq!(light.selection_active_sidebar, light.primary);
        let dark = Colors::dark();
        assert!((dark.selection_inactive.a - 0.2).abs() < 1e-6);
        assert!((dark.selection_active_sidebar.a - 0.3).abs() < 1e-6);
        assert!((dark.selection_active_note_list.a - 0.4).abs() < 1e-6);
    }

    #[test]
    fn the_dark_note_list_sits_between_the_page_and_the_sidebar() {
        let dark = Colors::dark();
        let page = dark.page_background.to_rgb().r;
        let sidebar = dark.sidebar_background.to_rgb().r;
        let note_list = dark.note_list_background.to_rgb().r;
        assert!(page < note_list && note_list < sidebar);
    }

    #[test]
    fn the_four_status_colours_are_distinct_and_shared_by_both_appearances() {
        let (light, dark) = (Colors::light(), Colors::dark());
        let statuses = [
            light.status_active,
            light.status_on_hold,
            light.status_completed,
            light.status_dropped,
        ];
        for (index, status) in statuses.iter().enumerate() {
            assert!(!statuses[index + 1..].contains(status));
        }
        assert_eq!(light.status_dropped, dark.status_dropped);
    }

    #[test]
    fn the_scale_holds_the_researched_metrics() {
        let scale = Scale::inkdrop();
        assert_eq!(scale.medium, px(14.));
        assert_eq!(scale.sidebar_row_height, px(30.));
        assert_eq!(scale.header_height, px(40.));
        assert_eq!(scale.search_bar_height, px(34.));
        assert_eq!(scale.sidebar_accent_bar, px(5.));
        assert_eq!(scale.splitter_handle, px(10.));
        assert_eq!(scale.sidebar_width.default, px(200.));
        assert_eq!(scale.sidebar_width.minimum, px(170.));
        assert_eq!(scale.note_list_width.default, px(260.));
        assert_eq!(scale.note_list_width.maximum, px(500.));
        assert_eq!(scale.transition, Duration::from_millis(100));
        assert_eq!(scale.transition_chevron, Duration::from_millis(200));
        assert!((scale.disabled_opacity - 0.45).abs() < f32::EPSILON);
    }

    #[test]
    fn the_type_scale_grows_strictly() {
        let scale = Scale::inkdrop();
        let sizes = [
            scale.mini,
            scale.tiny,
            scale.small,
            scale.medium,
            scale.large,
            scale.big,
            scale.huge,
            scale.massive,
        ];
        assert!(sizes.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn the_ui_font_is_inter_and_the_mono_stack_lists_it_first_choice_first() {
        let fonts = Fonts::bourbon();
        assert_eq!(fonts.ui_family.as_ref(), "Inter");
        assert_eq!(fonts.mono_stack[0], "SFMono-Regular");
        assert!(fonts.mono_stack.contains(&fonts.mono_family.as_ref()));
    }

    #[test]
    fn the_dark_shadows_are_stronger_than_the_light_ones() {
        let (light, dark) = (Tokens::light(), Tokens::dark());
        assert!(dark.shadows.floating[0].color.a > light.shadows.floating[0].color.a);
        assert_eq!(light.shadows.raised.len(), 3);
        assert_eq!(light.shadows.modal.len(), 6);
        let blurs: Vec<f32> = light
            .shadows
            .modal
            .iter()
            .map(|shadow| f32::from(shadow.blur_radius))
            .collect();
        assert!((blurs[0] - 2.8).abs() < 1e-6 && (blurs[5] - 100.).abs() < 1e-6);
        assert!(light.shadows.modal[0].color.a < light.shadows.modal[5].color.a);
        assert_eq!(light.shadows.palette[0].blur_radius, px(30.));
    }

    #[test]
    fn for_appearance_picks_the_matching_table() {
        assert_eq!(
            Tokens::for_appearance(Appearance::Dark)
                .colors
                .page_background,
            Colors::dark().page_background
        );
        assert_eq!(
            Tokens::for_appearance(Appearance::Light)
                .colors
                .page_background,
            Colors::light().page_background
        );
    }
}
