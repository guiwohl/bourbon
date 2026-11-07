use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    Action, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::icon::{Icon, IconName};
use crate::theme::ThemeExt;
use crate::tooltip::Tooltip;

pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Plain,
    Subtle,
    Primary,
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<IconName>,
    variant: ButtonVariant,
    disabled: bool,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    #[must_use]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            variant: ButtonVariant::Subtle,
            disabled: false,
            selected: false,
            on_click: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let (background, hover, down, text) = match self.variant {
            ButtonVariant::Plain => (
                gpui::transparent_black(),
                colors.transparent_black,
                colors.strong_transparent_black,
                colors.muted_text,
            ),
            ButtonVariant::Subtle => (
                colors.button_background,
                colors.button_hover_background,
                colors.button_down_background,
                colors.button_text,
            ),
            ButtonVariant::Primary => (
                colors.primary,
                colors.primary_hover,
                colors.primary_active,
                colors.inverted_text,
            ),
        };
        let background = if self.selected {
            colors.button_active_background
        } else {
            background
        };
        let mut content = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(tokens.scale.row_gap)
            .h(px(30.))
            .px(px(12.))
            .rounded(tokens.scale.radius_control)
            .bg(background)
            .text_color(text)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.medium)
            .font_weight(tokens.scale.weight_semibold)
            .when_some(self.icon, |row, icon| {
                row.child(Icon::new(icon).size(tokens.scale.icon_slot).color(text))
            })
            .children(self.label);
        if self.variant == ButtonVariant::Subtle {
            content = content.border_1().border_color(colors.button_ring);
        }
        if self.disabled {
            return content.opacity(tokens.scale.disabled_opacity);
        }
        content = content
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .active(move |style| style.bg(down));
        match self.on_click {
            Some(handler) => content.on_click(move |event, window, cx| handler(event, window, cx)),
            None => content,
        }
    }
}

pub(crate) enum TooltipSpec {
    Text(SharedString),
    Action(SharedString, Box<dyn Action>),
}

impl TooltipSpec {
    pub(crate) fn build(&self, window: &Window, cx: &mut App) -> gpui::AnyView {
        match self {
            TooltipSpec::Text(label) => Tooltip::text(label.clone(), cx),
            TooltipSpec::Action(label, action) => {
                Tooltip::for_action(label.clone(), action.as_ref(), window, cx).into()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconButtonSize {
    Header,
    Accessory,
}

#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    size: IconButtonSize,
    disabled: bool,
    selected: bool,
    tooltip: Option<Rc<TooltipSpec>>,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            size: IconButtonSize::Header,
            disabled: false,
            selected: false,
            tooltip: None,
            on_click: None,
        }
    }

    #[must_use]
    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub fn tooltip(mut self, label: impl Into<SharedString>) -> Self {
        self.tooltip = Some(Rc::new(TooltipSpec::Text(label.into())));
        self
    }

    #[must_use]
    pub fn tooltip_action(mut self, label: impl Into<SharedString>, action: &dyn Action) -> Self {
        self.tooltip = Some(Rc::new(TooltipSpec::Action(
            label.into(),
            action.boxed_clone(),
        )));
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
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let (side, glyph) = match self.size {
            IconButtonSize::Header => {
                (tokens.scale.icon_button_size, tokens.scale.icon_button_icon)
            }
            IconButtonSize::Accessory => {
                (tokens.scale.accessory_button_size, tokens.scale.icon_slot)
            }
        };
        let idle = if self.selected {
            colors.primary
        } else {
            colors.light_text
        };
        let hovered_icon = colors.hovered_text;
        let mut content = div()
            .id(self.id)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(side)
            .rounded(tokens.scale.radius_default)
            .text_color(idle)
            .child(Icon::new(self.icon).size(glyph));
        if self.selected {
            content = content.bg(colors.button_active_background);
        }
        if self.disabled {
            return content.opacity(tokens.scale.disabled_opacity);
        }
        content = content
            .cursor_pointer()
            .hover(move |style| {
                style
                    .bg(colors.button_hover_background)
                    .text_color(hovered_icon)
            })
            .active(move |style| style.bg(colors.button_down_background));
        if let Some(tooltip) = self.tooltip {
            content = content.tooltip(move |window, cx| tooltip.build(window, cx));
        }
        match self.on_click {
            Some(handler) => content.on_click(move |event, window, cx| handler(event, window, cx)),
            None => content,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{Modifiers, TestAppContext, point, px};

    use super::*;
    use crate::test_support::host;

    fn counter() -> (Rc<Cell<usize>>, Rc<Cell<usize>>) {
        let count = Rc::new(Cell::new(0));
        (count.clone(), count)
    }

    #[gpui::test]
    fn clicking_an_enabled_button_runs_its_handler_once(cx: &mut TestAppContext) {
        let (count, seen) = counter();
        let visual = host(cx, move || {
            let count = count.clone();
            Button::new("save")
                .label("Save")
                .on_click(move |_, _, _| count.set(count.get() + 1))
                .into_any_element()
        });

        visual.simulate_click(point(px(20.), px(10.)), Modifiers::none());

        assert_eq!(seen.get(), 1);
    }

    #[gpui::test]
    fn a_disabled_button_ignores_clicks(cx: &mut TestAppContext) {
        let (count, seen) = counter();
        let visual = host(cx, move || {
            let count = count.clone();
            Button::new("save")
                .label("Save")
                .disabled(true)
                .on_click(move |_, _, _| count.set(count.get() + 1))
                .into_any_element()
        });

        visual.simulate_click(point(px(20.), px(10.)), Modifiers::none());

        assert_eq!(seen.get(), 0);
    }

    #[gpui::test]
    fn clicking_an_icon_button_runs_its_handler(cx: &mut TestAppContext) {
        let (count, seen) = counter();
        let visual = host(cx, move || {
            let count = count.clone();
            IconButton::new("back", IconName::ArrowLeft)
                .tooltip("Back")
                .on_click(move |_, _, _| count.set(count.get() + 1))
                .into_any_element()
        });

        visual.simulate_click(point(px(15.), px(15.)), Modifiers::none());

        assert_eq!(seen.get(), 1);
    }

    #[gpui::test]
    fn a_disabled_icon_button_ignores_clicks(cx: &mut TestAppContext) {
        let (count, seen) = counter();
        let visual = host(cx, move || {
            let count = count.clone();
            IconButton::new("forward", IconName::ArrowRight)
                .disabled(true)
                .on_click(move |_, _, _| count.set(count.get() + 1))
                .into_any_element()
        });

        visual.simulate_click(point(px(15.), px(15.)), Modifiers::none());

        assert_eq!(seen.get(), 0);
    }
}
