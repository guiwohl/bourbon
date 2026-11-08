use bourbon_ui::{Button, ButtonVariant, IconButton, IconButtonSize, IconName, ThemeExt as _};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, ClickEvent, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use super::keys::{
    FindNext, FindPrevious, ReplaceAll, SEARCH_CONTEXT, ToggleMatchCase, ToggleRegex,
    ToggleWholeWord,
};
use super::panel::SearchPanel;

const PANEL_PADDING: f32 = 8.;
const ROW_GAP: f32 = 6.;
const BUTTON_COLUMN_WIDTH: f32 = 140.;
const COUNT_MIN_WIDTH_EM: f32 = 5.;

type PanelAction = fn(&mut SearchPanel, &mut Window, &mut Context<SearchPanel>);

pub(super) fn render(
    panel: &SearchPanel,
    _window: &mut Window,
    cx: &mut Context<SearchPanel>,
) -> impl IntoElement {
    if !panel.is_open() {
        return div();
    }
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    div()
        .key_context(SEARCH_CONTEXT)
        .flex()
        .flex_col()
        .gap(px(ROW_GAP))
        .p(px(PANEL_PADDING))
        .bg(colors.section_header_background)
        .border_b(tokens.scale.border)
        .border_color(colors.internal_border)
        .font_family(tokens.fonts.ui_family.clone())
        .text_size(tokens.scale.small)
        .on_action(cx.listener(|panel, _: &FindNext, _, cx| panel.find_next(cx)))
        .on_action(cx.listener(|panel, _: &FindPrevious, _, cx| panel.find_previous(cx)))
        .on_action(cx.listener(|panel, _: &ToggleMatchCase, _, cx| panel.toggle_match_case(cx)))
        .on_action(cx.listener(|panel, _: &ToggleRegex, _, cx| panel.toggle_regex(cx)))
        .on_action(cx.listener(|panel, _: &ToggleWholeWord, _, cx| panel.toggle_whole_word(cx)))
        .on_action(cx.listener(|panel, _: &ReplaceAll, _, cx| panel.replace_all(cx)))
        .child(find_row(panel, cx))
        .when(panel.is_replacing(), |root| {
            root.child(replace_row(panel, cx))
        })
}

fn find_row(panel: &SearchPanel, cx: &mut Context<SearchPanel>) -> impl IntoElement {
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    let count = panel.count_label(cx);
    let options = panel.options();
    div()
        .flex()
        .items_center()
        .gap(px(ROW_GAP))
        .child(div().flex_1().min_w_0().child(panel.find_field.clone()))
        .child(option_toggle(
            "search-match-case",
            "Aa",
            "Match case",
            options.match_case,
            |panel, _, cx| panel.toggle_match_case(cx),
            cx,
        ))
        .child(option_toggle(
            "search-whole-word",
            "ab",
            "Whole word",
            options.whole_word,
            |panel, _, cx| panel.toggle_whole_word(cx),
            cx,
        ))
        .child(option_toggle(
            "search-regex",
            ".*",
            "Regular expression",
            options.regex,
            |panel, _, cx| panel.toggle_regex(cx),
            cx,
        ))
        .child(
            div()
                .w(px(BUTTON_COLUMN_WIDTH))
                .flex_none()
                .flex()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .min_w(tokens.scale.small * COUNT_MIN_WIDTH_EM)
                        .text_color(colors.muted_text)
                        .child(count),
                )
                .child(
                    IconButton::new("search-previous", IconName::ChevronUp)
                        .size(IconButtonSize::Accessory)
                        .tooltip_action("Previous match", &FindPrevious)
                        .on_click(listen(|panel, _, cx| panel.find_previous(cx), cx)),
                )
                .child(
                    IconButton::new("search-next", IconName::ChevronDown)
                        .size(IconButtonSize::Accessory)
                        .tooltip_action("Next match", &FindNext)
                        .on_click(listen(|panel, _, cx| panel.find_next(cx), cx)),
                )
                .child(
                    IconButton::new("search-close", IconName::X)
                        .size(IconButtonSize::Accessory)
                        .tooltip("Close")
                        .on_click(listen(SearchPanel::close, cx)),
                ),
        )
}

fn replace_row(panel: &SearchPanel, cx: &mut Context<SearchPanel>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(ROW_GAP))
        .child(div().flex_1().min_w_0().child(panel.replace_field.clone()))
        .child(
            div()
                .w(px(BUTTON_COLUMN_WIDTH))
                .flex_none()
                .flex()
                .gap(px(ROW_GAP))
                .child(
                    Button::new("search-replace")
                        .label("Replace")
                        .variant(ButtonVariant::Subtle)
                        .on_click(listen(|panel, _, cx| panel.replace_current(cx), cx)),
                )
                .child(
                    Button::new("search-replace-all")
                        .label("All")
                        .variant(ButtonVariant::Subtle)
                        .on_click(listen(|panel, _, cx| panel.replace_all(cx), cx)),
                ),
        )
}

fn listen(
    action: PanelAction,
    cx: &mut Context<SearchPanel>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let panel = cx.entity().downgrade();
    move |_, window, cx| {
        panel.update(cx, |panel, cx| action(panel, window, cx)).ok();
    }
}

fn option_toggle(
    id: &'static str,
    glyph: &'static str,
    label: &'static str,
    enabled: bool,
    action: PanelAction,
    cx: &mut Context<SearchPanel>,
) -> impl IntoElement {
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    let (text, background) = if enabled {
        (colors.primary, colors.button_active_background)
    } else {
        (colors.light_text, gpui::transparent_black())
    };
    let hover = colors.transparent_black;
    let tooltip = SharedString::from(label);
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(tokens.scale.accessory_button_size)
        .rounded(tokens.scale.radius_default)
        .font_family(tokens.fonts.mono_family.clone())
        .text_size(tokens.scale.tiny)
        .text_color(text)
        .bg(background)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .tooltip(move |_, cx| bourbon_ui::Tooltip::text(tooltip.clone(), cx))
        .on_click(listen(action, cx))
        .child(glyph)
}
