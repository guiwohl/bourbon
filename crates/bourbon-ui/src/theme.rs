use std::panic::Location;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};

pub use crate::tokens::Appearance;
use crate::tokens::Tokens;

#[derive(Clone, Debug)]
pub struct Theme {
    pub tokens: Tokens,
    pub appearance: Appearance,
}

impl gpui::Global for Theme {}

impl Theme {
    #[must_use]
    pub fn new(appearance: Appearance) -> Self {
        Self {
            tokens: Tokens::for_appearance(appearance),
            appearance,
        }
    }

    #[must_use]
    pub fn with_tokens(tokens: Tokens, appearance: Appearance) -> Self {
        Self { tokens, appearance }
    }

    #[must_use]
    pub fn is_dark(&self) -> bool {
        self.appearance == Appearance::Dark
    }

    pub fn install(appearance: Appearance, cx: &mut App) {
        cx.set_global(Self::new(appearance));
    }

    pub fn install_tokens(tokens: Tokens, appearance: Appearance, cx: &mut App) {
        cx.set_global(Self::with_tokens(tokens, appearance));
    }
}

pub trait ThemeExt {
    fn theme(&self) -> &Theme;
}

impl ThemeExt for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

pub struct Themed {
    appearance: Appearance,
    child: AnyElement,
}

impl Themed {
    #[must_use]
    pub fn new(appearance: Appearance, child: impl IntoElement) -> Self {
        Self {
            appearance,
            child: child.into_any_element(),
        }
    }
}

fn within_scope<T>(appearance: Appearance, cx: &mut App, work: impl FnOnce(&mut App) -> T) -> T {
    let previous = cx.theme().clone();
    cx.set_global(Theme::new(appearance));
    let result = work(cx);
    cx.set_global(previous);
    result
}

impl IntoElement for Themed {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for Themed {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let layout = within_scope(self.appearance, cx, |cx| {
            self.child.request_layout(window, cx)
        });
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        within_scope(self.appearance, cx, |cx| {
            self.child.prepaint(window, cx);
        });
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{
        Context, InteractiveElement as _, IntoElement, Render, RenderOnce,
        StatefulInteractiveElement as _, Styled as _, TestAppContext, div,
    };

    use super::*;

    #[derive(IntoElement)]
    struct Probe {
        seen: Rc<Cell<Option<Appearance>>>,
    }

    impl RenderOnce for Probe {
        fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
            self.seen.set(Some(cx.theme().appearance));
            div()
        }
    }

    struct Scoped {
        seen: Rc<Cell<Option<Appearance>>>,
    }

    impl Render for Scoped {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Themed::new(
                Appearance::Dark,
                Probe {
                    seen: self.seen.clone(),
                },
            )
        }
    }

    #[gpui::test]
    fn components_inside_a_themed_scope_see_that_appearance_and_the_global_is_restored(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| Theme::install(Appearance::Light, cx));
        let seen = Rc::new(Cell::new(None));
        let probe_seen = seen.clone();
        let (_, visual) = cx.add_window_view(|_, _| Scoped { seen: probe_seen });
        visual.update(|window, _| window.refresh());
        visual.run_until_parked();

        assert_eq!(seen.get(), Some(Appearance::Dark));
        assert_eq!(
            visual.update(|_, cx| cx.theme().appearance),
            Appearance::Light
        );
    }

    #[gpui::test]
    fn a_themed_child_is_painted_and_clickable(cx: &mut TestAppContext) {
        let clicked = Rc::new(Cell::new(false));
        let seen = clicked.clone();
        let visual = crate::test_support::host(cx, move || {
            let seen = seen.clone();
            Themed::new(
                Appearance::Dark,
                div()
                    .id("inside")
                    .size(gpui::px(50.))
                    .on_click(move |_, _, _| seen.set(true)),
            )
            .into_any_element()
        });

        visual.simulate_click(
            gpui::point(gpui::px(10.), gpui::px(10.)),
            gpui::Modifiers::none(),
        );

        assert!(clicked.get());
    }

    #[test]
    fn a_theme_can_carry_custom_tokens_for_its_appearance() {
        let mut tokens = Tokens::dark();
        tokens.colors.primary = gpui::red();

        let theme = Theme::with_tokens(tokens, Appearance::Dark);

        assert!(theme.is_dark());
        assert_eq!(theme.tokens.colors.primary, gpui::red());
    }

    #[gpui::test]
    fn installing_tokens_replaces_the_global_theme(cx: &mut TestAppContext) {
        cx.update(|cx| Theme::install(Appearance::Light, cx));
        let mut tokens = Tokens::dark();
        tokens.colors.primary = gpui::red();

        cx.update(|cx| Theme::install_tokens(tokens, Appearance::Dark, cx));

        cx.update(|cx| {
            assert_eq!(cx.theme().appearance, Appearance::Dark);
            assert_eq!(cx.theme().tokens.colors.primary, gpui::red());
        });
    }

    #[test]
    fn a_new_theme_carries_the_tokens_of_its_appearance() {
        assert!(Theme::new(Appearance::Dark).is_dark());
        assert!(!Theme::new(Appearance::Light).is_dark());
        assert_ne!(
            Theme::new(Appearance::Dark).tokens.colors.page_background,
            Theme::new(Appearance::Light).tokens.colors.page_background
        );
    }
}
