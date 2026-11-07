use gpui::{
    Action, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Window, div, px,
};

use crate::theme::ThemeExt;

pub struct Tooltip {
    label: SharedString,
    binding: Option<SharedString>,
}

#[must_use]
pub fn summary(label: &str, binding: Option<&str>) -> String {
    match binding {
        Some(binding) => format!("{label} ({binding})"),
        None => label.to_owned(),
    }
}

#[must_use]
pub fn live_binding(action: &dyn Action, window: &Window, cx: &App) -> Option<String> {
    let os = cx.platform_info().os;
    window.bindings_for_action(action).last().map(|binding| {
        binding
            .keystrokes()
            .iter()
            .map(|keystroke| keystroke.display(os).to_string())
            .collect::<Vec<_>>()
            .join(" ")
    })
}

impl Tooltip {
    pub fn new(
        label: impl Into<SharedString>,
        binding: Option<SharedString>,
        cx: &mut App,
    ) -> Entity<Self> {
        let label = label.into();
        cx.new(|_| Self { label, binding })
    }

    pub fn text(label: impl Into<SharedString>, cx: &mut App) -> AnyView {
        Self::new(label, None, cx).into()
    }

    pub fn for_action(
        label: impl Into<SharedString>,
        action: &dyn Action,
        window: &Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let binding = live_binding(action, window, cx).map(SharedString::from);
        Self::new(label, binding, cx)
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub fn binding(&self) -> Option<&str> {
        self.binding.as_deref()
    }

    #[must_use]
    pub fn summary(&self) -> String {
        summary(&self.label, self.binding())
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = &cx.theme().tokens;
        let colors = &tokens.colors;
        let kbd = self.binding.clone().map(|binding| {
            div()
                .px(px(5.))
                .py(px(2.))
                .rounded(px(6.))
                .border_1()
                .border_color(colors.kbd_border)
                .bg(colors.kbd_background)
                .text_color(colors.muted_text)
                .child(binding)
        });
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(4.))
            .px(tokens.scale.tooltip_padding_horizontal)
            .py(tokens.scale.tooltip_padding_vertical)
            .bg(colors.menu_background)
            .border_1()
            .border_color(colors.solid_border)
            .rounded(tokens.scale.radius_control)
            .shadow(tokens.shadows.floating.clone())
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.mini)
            .text_color(colors.muted_text)
            .child(self.label.clone())
            .children(kbd)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{KeyBinding, TestAppContext, actions};

    use super::*;
    use crate::test_support::prepare;

    actions!(tooltip_tests, [Rename, Unbound]);

    #[test]
    fn the_summary_appends_the_binding_in_parentheses() {
        assert_eq!(summary("Back", Some("alt-left")), "Back (alt-left)");
        assert_eq!(summary("Back", None), "Back");
    }

    #[gpui::test]
    fn an_action_tooltip_shows_the_label_and_its_live_keybinding(cx: &mut TestAppContext) {
        prepare(cx);
        cx.update(|cx| cx.bind_keys([KeyBinding::new("ctrl-shift-r", Rename, None)]));
        let (_, visual) = cx.add_window_view(|_, _| {
            crate::test_support::Host::new(|| gpui::div().into_any_element())
        });

        let tooltip = visual.update(|window, cx| {
            let tooltip = Tooltip::for_action("Rename note", &Rename, window, cx);
            let tooltip = tooltip.read(cx);
            (
                tooltip.label().to_owned(),
                tooltip.binding().map(str::to_owned),
                tooltip.summary(),
            )
        });
        assert_eq!(tooltip.0, "Rename note");
        let binding = tooltip
            .1
            .expect("the bound action shows a binding")
            .to_lowercase();
        assert!(
            binding.contains("ctrl") && binding.contains("shift") && binding.contains('r'),
            "{binding}"
        );
        assert!(tooltip.2.starts_with("Rename note ("));
        assert!(tooltip.2.to_lowercase().contains(&binding));
    }

    #[gpui::test]
    fn the_tooltip_follows_a_rebinding(cx: &mut TestAppContext) {
        prepare(cx);
        cx.update(|cx| cx.bind_keys([KeyBinding::new("ctrl-r", Rename, None)]));
        let (_, visual) = cx.add_window_view(|_, _| {
            crate::test_support::Host::new(|| gpui::div().into_any_element())
        });
        visual.update(|_, cx| cx.bind_keys([KeyBinding::new("ctrl-alt-q", Rename, None)]));

        let binding = visual
            .update(|window, cx| live_binding(&Rename, window, cx))
            .unwrap()
            .to_lowercase();

        assert!(
            binding.contains("alt") && binding.contains('q'),
            "{binding}"
        );
    }

    #[gpui::test]
    fn an_unbound_action_tooltip_shows_only_the_label(cx: &mut TestAppContext) {
        prepare(cx);
        let (_, visual) = cx.add_window_view(|_, _| {
            crate::test_support::Host::new(|| gpui::div().into_any_element())
        });

        let (binding, summary) = visual.update(|window, cx| {
            let tooltip = Tooltip::for_action("Nothing", &Unbound, window, cx);
            let tooltip = tooltip.read(cx);
            (tooltip.binding().map(str::to_owned), tooltip.summary())
        });

        assert_eq!(binding, None);
        assert_eq!(summary, "Nothing");
    }
}
