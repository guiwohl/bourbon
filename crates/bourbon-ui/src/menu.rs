use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, Context, ElementId, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, actions, div, px,
};

use crate::icon::{Icon, IconName};
use crate::navigation::{Step, next_enabled};
use crate::theme::ThemeExt;
use crate::tokens::Tokens;

actions!(
    bourbon_menu,
    [
        SelectNext,
        SelectPrevious,
        SelectFirst,
        SelectLast,
        Confirm,
        Cancel
    ]
);

pub const KEY_CONTEXT: &str = "BourbonMenu";

const ITEM_HEIGHT: f32 = 36.;
const SEPARATOR_MARGIN: f32 = 7.;
const SLOT: f32 = 16.;
const SHORTCUT_OPACITY: f32 = 0.4;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, Some(KEY_CONTEXT)),
        KeyBinding::new("up", SelectPrevious, Some(KEY_CONTEXT)),
        KeyBinding::new("home", SelectFirst, Some(KEY_CONTEXT)),
        KeyBinding::new("end", SelectLast, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Confirm, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(KEY_CONTEXT)),
    ]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub shortcut: Option<SharedString>,
    pub checked: bool,
    pub disabled: bool,
    pub submenu: bool,
}

impl MenuItem {
    #[must_use]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            shortcut: None,
            checked: false,
            disabled: false,
            submenu: false,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn submenu(mut self) -> Self {
        self.submenu = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEvent {
    Confirmed(usize),
    Dismissed,
}

pub struct Menu {
    entries: Vec<MenuEntry>,
    selected: Option<usize>,
    focus_handle: FocusHandle,
}

impl EventEmitter<MenuEvent> for Menu {}

impl Focusable for Menu {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn is_selected(selected: Option<usize>, index: usize) -> bool {
    selected == Some(index)
}

fn after_hover(hovered: bool, selected: Option<usize>, index: usize) -> Option<usize> {
    if hovered { Some(index) } else { selected }
}

fn enabled_flags(entries: &[MenuEntry]) -> Vec<bool> {
    entries
        .iter()
        .map(|entry| matches!(entry, MenuEntry::Item(item) if !item.disabled))
        .collect()
}

impl Menu {
    pub fn new(entries: Vec<MenuEntry>, cx: &mut Context<Self>) -> Self {
        Self {
            entries,
            selected: None,
            focus_handle: cx.focus_handle(),
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[MenuEntry] {
        &self.entries
    }

    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    fn move_selection(&mut self, step: Step, cx: &mut Context<Self>) {
        self.selected = next_enabled(&enabled_flags(&self.entries), self.selected, step);
        cx.notify();
    }

    fn select_edge(&mut self, step: Step, cx: &mut Context<Self>) {
        self.selected = next_enabled(&enabled_flags(&self.entries), None, step);
        cx.notify();
    }

    fn confirm_at(&mut self, index: usize, cx: &mut Context<Self>) {
        if enabled_flags(&self.entries).get(index) == Some(&true) {
            cx.emit(MenuEvent::Confirmed(index));
        }
    }

    fn item_row(
        &self,
        index: usize,
        item: &MenuItem,
        any_icon: bool,
        tokens: &Tokens,
        cx: &Context<Self>,
    ) -> AnyElement {
        let colors = tokens.colors;
        let is_selected = is_selected(self.selected, index);
        let text = if item.disabled {
            colors.disabled_text
        } else {
            colors.text
        };
        div()
            .id(ElementId::from(("menu-item", index)))
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(ITEM_HEIGHT))
            .px(tokens.scale.menu_item_padding_horizontal)
            .text_color(text)
            .when(is_selected, |row| row.bg(colors.menu_active))
            .when(!item.disabled, |row| {
                row.cursor_pointer()
                    .on_hover(cx.listener(move |menu, hovered: &bool, _, cx| {
                        menu.selected = after_hover(*hovered, menu.selected, index);
                        cx.notify();
                    }))
                    .on_click(cx.listener(move |menu, _, _, cx| menu.confirm_at(index, cx)))
            })
            .child(div().flex_none().size(px(SLOT)).when(item.checked, |slot| {
                slot.child(Icon::new(IconName::Check).size(px(SLOT)).color(text))
            }))
            .when(any_icon, |row| {
                row.child(
                    div()
                        .flex_none()
                        .size(px(SLOT))
                        .when_some(item.icon, |slot, icon| {
                            slot.child(Icon::new(icon).size(px(SLOT)).color(text))
                        }),
                )
            })
            .child(div().flex_1().whitespace_nowrap().child(item.label.clone()))
            .when_some(item.shortcut.clone(), |row, shortcut| {
                row.child(
                    div()
                        .flex_none()
                        .ml(px(24.))
                        .text_size(tokens.scale.tiny)
                        .opacity(SHORTCUT_OPACITY)
                        .child(shortcut),
                )
            })
            .when(item.submenu, |row| {
                row.child(
                    Icon::new(IconName::ChevronRight)
                        .size(px(12.))
                        .color(colors.muted_text),
                )
            })
            .into_any_element()
    }
}

impl Render for Menu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let any_icon = self
            .entries
            .iter()
            .any(|entry| matches!(entry, MenuEntry::Item(item) if item.icon.is_some()));
        let rows = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| match entry {
                MenuEntry::Separator => div()
                    .my(px(SEPARATOR_MARGIN))
                    .h(tokens.scale.border)
                    .bg(colors.internal_border)
                    .into_any_element(),
                MenuEntry::Item(item) => self.item_row(index, item, any_icon, &tokens, cx),
            })
            .collect::<Vec<_>>();
        div()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|menu, _: &SelectNext, _, cx| {
                menu.move_selection(Step::Forward, cx);
            }))
            .on_action(cx.listener(|menu, _: &SelectPrevious, _, cx| {
                menu.move_selection(Step::Backward, cx);
            }))
            .on_action(cx.listener(|menu, _: &SelectFirst, _, cx| {
                menu.select_edge(Step::Forward, cx);
            }))
            .on_action(cx.listener(|menu, _: &SelectLast, _, cx| {
                menu.select_edge(Step::Backward, cx);
            }))
            .on_action(cx.listener(|menu, _: &Confirm, _, cx| {
                if let Some(index) = menu.selected {
                    menu.confirm_at(index, cx);
                }
            }))
            .on_action(cx.listener(|_, _: &Cancel, _, cx| cx.emit(MenuEvent::Dismissed)))
            .flex()
            .flex_col()
            .min_w(tokens.scale.menu_min_width)
            .bg(colors.menu_background)
            .border_1()
            .border_color(colors.border)
            .rounded(tokens.scale.radius_control)
            .shadow(tokens.shadows.raised.clone())
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.medium)
            .children(rows)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::test_support::prepare;

    fn entries() -> Vec<MenuEntry> {
        vec![
            MenuEntry::Item(MenuItem::new("Rename")),
            MenuEntry::Item(MenuItem::new("Move").disabled(true)),
            MenuEntry::Separator,
            MenuEntry::Item(MenuItem::new("Delete").shortcut("Del")),
        ]
    }

    struct Opened<'a> {
        menu: Entity<Menu>,
        visual: &'a mut VisualTestContext,
        events: Rc<RefCell<Vec<MenuEvent>>>,
    }

    fn open(cx: &mut TestAppContext) -> Opened<'_> {
        prepare(cx);
        let events = Rc::new(RefCell::new(Vec::new()));
        let (menu, visual) = cx.add_window_view(|_, cx| Menu::new(entries(), cx));
        let recorded = events.clone();
        visual.update(|window, cx| {
            cx.subscribe(&menu, move |_, event: &MenuEvent, _| {
                recorded.borrow_mut().push(*event);
            })
            .detach();
            let entity = menu.clone();
            entity.update(cx, |menu, cx| menu.focus(window, cx));
        });
        visual.run_until_parked();
        Opened {
            menu,
            visual,
            events,
        }
    }

    fn selected(opened: &mut Opened<'_>) -> Option<usize> {
        opened
            .visual
            .read_entity(&opened.menu, |menu, _| menu.selected())
    }

    #[gpui::test]
    fn the_menu_exposes_its_entries_in_order(cx: &mut TestAppContext) {
        let opened = open(cx);

        let labels: Vec<_> = opened
            .menu
            .read_with(opened.visual, |menu, _| menu.entries().to_vec());

        assert_eq!(labels, entries());
    }

    #[gpui::test]
    fn down_selects_enabled_items_skipping_disabled_ones_and_separators(cx: &mut TestAppContext) {
        let mut opened = open(cx);

        opened.visual.simulate_keystrokes("down");
        assert_eq!(selected(&mut opened), Some(0));
        opened.visual.simulate_keystrokes("down");
        assert_eq!(selected(&mut opened), Some(3));
    }

    #[gpui::test]
    fn down_wraps_from_the_last_item_to_the_first(cx: &mut TestAppContext) {
        let mut opened = open(cx);

        opened.visual.simulate_keystrokes("down down down");

        assert_eq!(selected(&mut opened), Some(0));
    }

    #[gpui::test]
    fn up_from_nothing_selects_the_last_enabled_item(cx: &mut TestAppContext) {
        let mut opened = open(cx);

        opened.visual.simulate_keystrokes("up");
        assert_eq!(selected(&mut opened), Some(3));
        opened.visual.simulate_keystrokes("up");
        assert_eq!(selected(&mut opened), Some(0));
    }

    #[gpui::test]
    fn home_and_end_jump_to_the_first_and_last_enabled_items(cx: &mut TestAppContext) {
        let mut opened = open(cx);

        opened.visual.simulate_keystrokes("end");
        assert_eq!(selected(&mut opened), Some(3));
        opened.visual.simulate_keystrokes("home");
        assert_eq!(selected(&mut opened), Some(0));
    }

    #[gpui::test]
    fn enter_confirms_the_selected_item(cx: &mut TestAppContext) {
        let opened = open(cx);

        opened.visual.simulate_keystrokes("down down enter");

        assert_eq!(*opened.events.borrow(), [MenuEvent::Confirmed(3)]);
    }

    #[gpui::test]
    fn enter_with_nothing_selected_confirms_nothing(cx: &mut TestAppContext) {
        let opened = open(cx);

        opened.visual.simulate_keystrokes("enter");

        assert!(opened.events.borrow().is_empty());
    }

    #[gpui::test]
    fn escape_dismisses_the_menu(cx: &mut TestAppContext) {
        let opened = open(cx);

        opened.visual.simulate_keystrokes("down escape");

        assert_eq!(*opened.events.borrow(), [MenuEvent::Dismissed]);
    }

    #[gpui::test]
    fn clicking_an_item_confirms_it_and_clicking_a_disabled_one_does_not(cx: &mut TestAppContext) {
        let opened = open(cx);

        opened
            .visual
            .simulate_click(point(px(60.), px(55.)), Modifiers::none());
        assert!(opened.events.borrow().is_empty());
        opened
            .visual
            .simulate_click(point(px(60.), px(20.)), Modifiers::none());

        assert_eq!(*opened.events.borrow(), [MenuEvent::Confirmed(0)]);
    }

    #[gpui::test]
    fn hovering_an_item_moves_the_selection_to_it(cx: &mut TestAppContext) {
        let mut opened = open(cx);

        opened
            .visual
            .simulate_mouse_move(point(px(60.), px(20.)), None, Modifiers::none());

        assert_eq!(selected(&mut opened), Some(0));
    }

    #[gpui::test]
    fn leaving_an_item_with_the_mouse_keeps_the_keyboard_selection(cx: &mut TestAppContext) {
        let mut opened = open(cx);

        opened
            .visual
            .simulate_mouse_move(point(px(60.), px(20.)), None, Modifiers::none());
        opened.visual.simulate_keystrokes("down");
        opened
            .visual
            .simulate_mouse_move(point(px(900.), px(600.)), None, Modifiers::none());

        assert_eq!(selected(&mut opened), Some(3));
    }

    #[test]
    fn selection_and_hover_rules() {
        assert!(is_selected(Some(2), 2));
        assert!(!is_selected(Some(2), 1));
        assert!(!is_selected(None, 0));
        assert_eq!(after_hover(true, Some(1), 3), Some(3));
        assert_eq!(after_hover(false, Some(1), 3), Some(1));
        assert_eq!(after_hover(false, None, 3), None);
    }

    #[test]
    fn only_items_that_are_not_disabled_are_enabled() {
        assert_eq!(enabled_flags(&entries()), [true, false, false, true]);
    }

    #[gpui::test]
    fn a_menu_builds_from_entries(cx: &mut TestAppContext) {
        let menu = cx.new(|cx| Menu::new(entries(), cx));
        assert_eq!(menu.read_with(cx, |menu, _| menu.selected()), None);
    }
}
