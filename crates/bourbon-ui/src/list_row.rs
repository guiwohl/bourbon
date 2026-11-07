use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    Animation, AnimationExt as _, AnyElement, App, ClickEvent, Div, ElementId, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, Window, div, ease_out_quint, px,
};

use crate::button::{ClickHandler, IconButton, IconButtonSize};
use crate::icon::{Icon, IconName, Stroke};
use crate::theme::ThemeExt;
use crate::tokens::Tokens;

type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

const COUNT_SIZE: f32 = 10.;
const COUNT_OPACITY: f32 = 0.6;
const DISCLOSURE_SIZE: f32 = 12.;
const DISCLOSURE_IDLE_OPACITY: f32 = 0.5;
const DISCLOSURE_HOVER_OPACITY: f32 = 0.8;

#[must_use]
pub fn chevron_turns(expanded: bool, progress: f32) -> f32 {
    if expanded { progress } else { 1. - progress }
}

const BASE_INDENT: f32 = 14.;
const INDENT_STEP: f32 = 12.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowAccent {
    pub tinted: bool,
    pub bar: bool,
}

#[must_use]
pub fn accent(selected: bool, pane_focused: bool) -> RowAccent {
    RowAccent {
        tinted: selected,
        bar: selected && pane_focused,
    }
}

#[must_use]
pub fn indent_padding(depth: usize) -> f32 {
    BASE_INDENT + INDENT_STEP * f32::from(u8::try_from(depth).unwrap_or(u8::MAX))
}

fn disclosure_toggle(
    row_id: &ElementId,
    expanded: bool,
    toggle: Option<ToggleHandler>,
    tokens: &Tokens,
) -> Stateful<Div> {
    let chevron = Icon::new(IconName::ChevronRight)
        .size(px(DISCLOSURE_SIZE))
        .stroke(Stroke::Heavy)
        .with_animation(
            ElementId::from((
                SharedString::from(format!("{row_id:?}-chevron-{expanded}")),
                0usize,
            )),
            Animation::new(tokens.scale.transition_chevron).with_easing(ease_out_quint()),
            move |icon, progress| icon.quarter_turns(chevron_turns(expanded, progress)),
        );
    div()
        .id(ElementId::from((
            SharedString::from(format!("{row_id:?}-toggle")),
            0usize,
        )))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(DISCLOSURE_SIZE))
        .opacity(DISCLOSURE_IDLE_OPACITY)
        .hover(|style| style.opacity(DISCLOSURE_HOVER_OPACITY))
        .child(chevron)
        .when_some(toggle, |element, toggle| {
            element.on_click(move |_, window, cx| {
                cx.stop_propagation();
                toggle(!expanded, window, cx);
            })
        })
}

#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    icon_color: Option<Hsla>,
    count: Option<usize>,
    depth: usize,
    disclosure: Option<bool>,
    selected: bool,
    pane_focused: bool,
    on_click: Option<ClickHandler>,
    on_toggle: Option<ToggleHandler>,
    accessories: Vec<AnyElement>,
}

impl ListRow {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            icon_color: None,
            count: None,
            depth: 0,
            disclosure: None,
            selected: false,
            pane_focused: false,
            on_click: None,
            on_toggle: None,
            accessories: Vec::new(),
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn icon_color(mut self, color: Hsla) -> Self {
        self.icon_color = Some(color);
        self
    }

    #[must_use]
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    #[must_use]
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    #[must_use]
    pub fn disclosure(mut self, expanded: bool) -> Self {
        self.disclosure = Some(expanded);
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub fn pane_focused(mut self, pane_focused: bool) -> Self {
        self.pane_focused = pane_focused;
        self
    }

    #[must_use]
    pub fn accessory(mut self, accessory: impl IntoElement) -> Self {
        self.accessories.push(accessory.into_any_element());
        self
    }

    #[must_use]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ListRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let shown = accent(self.selected, self.pane_focused);
        let text = if self.selected {
            colors.sidebar_item_active_text
        } else {
            colors.sidebar_item_text
        };
        let group = SharedString::from(format!("{:?}-row", self.id));
        let disclosure = self
            .disclosure
            .map(|expanded| disclosure_toggle(&self.id, expanded, self.on_toggle.clone(), &tokens));
        let icon_color = self.icon_color.unwrap_or(text);
        let row = div()
            .id(self.id)
            .group(group.clone())
            .relative()
            .flex()
            .items_center()
            .gap(tokens.scale.row_gap)
            .min_h(tokens.scale.sidebar_row_height)
            .pl(px(indent_padding(self.depth)))
            .pr(tokens.scale.sidebar_row_trailing_padding)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.medium)
            .text_color(text)
            .when(shown.tinted, |row| row.bg(colors.selection_inactive))
            .when(shown.bar, |row| {
                row.child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .w(tokens.scale.sidebar_accent_bar)
                        .bg(colors.selection_active_sidebar),
                )
            })
            .children(disclosure)
            .when_some(self.icon, |row, icon| {
                row.child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(tokens.scale.icon_slot)
                        .child(
                            Icon::new(icon)
                                .size(tokens.scale.icon_slot)
                                .color(icon_color),
                        ),
                )
            })
            .child(div().flex_1().min_w_0().truncate().child(self.label))
            .when_some(self.count, |row, count| {
                row.child(
                    div()
                        .mx(px(4.))
                        .text_size(px(COUNT_SIZE))
                        .font_weight(FontWeight::EXTRA_LIGHT)
                        .opacity(COUNT_OPACITY)
                        .child(count.to_string()),
                )
            })
            .when(!self.accessories.is_empty(), |row| {
                row.child(
                    div()
                        .flex()
                        .items_center()
                        .invisible()
                        .group_hover(group, gpui::Styled::visible)
                        .children(self.accessories),
                )
            });
        match self.on_click {
            Some(handler) => row.on_click(move |event, window, cx| handler(event, window, cx)),
            None => row,
        }
    }
}

type SectionToggle = Rc<dyn Fn(bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct SectionHeader {
    id: ElementId,
    label: SharedString,
    collapsed: Option<bool>,
    on_collapse: Option<SectionToggle>,
    accessories: Vec<AnyElement>,
}

impl SectionHeader {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            collapsed: None,
            on_collapse: None,
            accessories: Vec::new(),
        }
    }

    #[must_use]
    pub fn accessory(mut self, accessory: impl IntoElement) -> Self {
        self.accessories.push(accessory.into_any_element());
        self
    }

    #[must_use]
    pub fn collapsible(
        mut self,
        collapsed: bool,
        on_collapse: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.collapsed = Some(collapsed);
        self.on_collapse = Some(Rc::new(on_collapse));
        self
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let collapse = self.collapsed.map(|collapsed| {
            let handler = self.on_collapse.clone();
            let icon = if collapsed {
                IconName::ChevronDown
            } else {
                IconName::ChevronUp
            };
            IconButton::new(
                ElementId::from((
                    SharedString::from(format!("{:?}-collapse", self.id)),
                    0usize,
                )),
                icon,
            )
            .size(IconButtonSize::Accessory)
            .on_click(move |_, window, cx| {
                if let Some(handler) = &handler {
                    handler(!collapsed, window, cx);
                }
            })
        });
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(tokens.scale.row_gap)
            .min_h(tokens.scale.sidebar_row_height)
            .pl(px(indent_padding(0)))
            .pr(tokens.scale.sidebar_row_trailing_padding)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.tiny)
            .text_color(tokens.colors.sidebar_section_text)
            .child(div().flex_1().min_w_0().truncate().child(self.label))
            .children(self.accessories)
            .children(collapse)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Modifiers, TestAppContext, point};

    use super::*;
    use crate::test_support::host;

    #[test]
    fn an_unselected_row_has_no_tint_and_no_bar_even_when_the_pane_is_focused() {
        assert_eq!(
            accent(false, true),
            RowAccent {
                tinted: false,
                bar: false
            }
        );
        assert_eq!(
            accent(false, false),
            RowAccent {
                tinted: false,
                bar: false
            }
        );
    }

    #[test]
    fn a_selected_row_in_an_unfocused_pane_is_tinted_without_the_bar() {
        assert_eq!(
            accent(true, false),
            RowAccent {
                tinted: true,
                bar: false
            }
        );
    }

    #[test]
    fn a_selected_row_in_a_focused_pane_is_tinted_with_the_bar() {
        assert_eq!(
            accent(true, true),
            RowAccent {
                tinted: true,
                bar: true
            }
        );
    }

    #[test]
    fn indentation_starts_at_fourteen_and_grows_per_depth() {
        assert!((indent_padding(0) - 14.).abs() < f32::EPSILON);
        assert!((indent_padding(2) - 38.).abs() < f32::EPSILON);
    }

    #[test]
    fn the_chevron_points_down_when_expanded_and_right_when_collapsed() {
        assert!((chevron_turns(true, 1.) - 1.).abs() < f32::EPSILON);
        assert!(chevron_turns(false, 1.).abs() < f32::EPSILON);
        assert!((chevron_turns(true, 0.25) - 0.25).abs() < f32::EPSILON);
        assert!((chevron_turns(false, 0.25) - 0.75).abs() < f32::EPSILON);
    }

    fn sidebar_row(
        clicks: Rc<RefCell<Vec<&'static str>>>,
        expanded: bool,
    ) -> impl Fn() -> AnyElement {
        move || {
            let (on_click, on_toggle) = (clicks.clone(), clicks.clone());
            div()
                .w(px(200.))
                .child(
                    ListRow::new("notebook-work", "Work")
                        .icon(IconName::Notebook)
                        .count(4)
                        .disclosure(expanded)
                        .on_click(move |_, _, _| on_click.borrow_mut().push("row"))
                        .on_toggle(move |opens, _, _| {
                            on_toggle
                                .borrow_mut()
                                .push(if opens { "expand" } else { "collapse" });
                        }),
                )
                .into_any_element()
        }
    }

    #[gpui::test]
    fn clicking_the_label_area_selects_the_row_only(cx: &mut TestAppContext) {
        let clicks = Rc::new(RefCell::new(Vec::new()));
        let visual = host(cx, sidebar_row(clicks.clone(), false));

        visual.simulate_click(point(px(100.), px(15.)), Modifiers::none());

        assert_eq!(*clicks.borrow(), ["row"]);
    }

    #[gpui::test]
    fn clicking_the_disclosure_toggles_without_selecting_the_row(cx: &mut TestAppContext) {
        let clicks = Rc::new(RefCell::new(Vec::new()));
        let visual = host(cx, sidebar_row(clicks.clone(), false));

        visual.simulate_click(point(px(20.), px(15.)), Modifiers::none());

        assert_eq!(*clicks.borrow(), ["expand"]);
    }

    #[gpui::test]
    fn clicking_the_disclosure_of_an_expanded_row_collapses_it(cx: &mut TestAppContext) {
        let clicks = Rc::new(RefCell::new(Vec::new()));
        let visual = host(cx, sidebar_row(clicks.clone(), true));

        visual.simulate_click(point(px(20.), px(15.)), Modifiers::none());

        assert_eq!(*clicks.borrow(), ["collapse"]);
    }

    #[gpui::test]
    fn the_section_collapse_button_asks_for_the_opposite_state(cx: &mut TestAppContext) {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let recorded = requests.clone();
        let visual = host(cx, move || {
            let recorded = recorded.clone();
            div()
                .w(px(200.))
                .child(SectionHeader::new("tags", "Tags").collapsible(
                    false,
                    move |collapsed, _, _| {
                        recorded.borrow_mut().push(collapsed);
                    },
                ))
                .into_any_element()
        });

        visual.simulate_click(point(px(183.), px(15.)), Modifiers::none());

        assert_eq!(*requests.borrow(), [true]);
    }
}
