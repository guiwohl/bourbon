use std::cell::Cell;
use std::rc::Rc;

use gpui::{Bounds, Div, IntoElement, ParentElement as _, Pixels, Point, div, point, px};

const MENU_GAP: f32 = 4.;

#[derive(Clone, Default)]
pub(crate) struct MenuAnchor(Rc<Cell<Bounds<Pixels>>>);

#[derive(Default)]
pub(crate) struct MenuAnchors {
    pub(crate) notebook: MenuAnchor,
    pub(crate) status: MenuAnchor,
    pub(crate) more: MenuAnchor,
}

impl MenuAnchor {
    pub(crate) fn below(&self) -> Point<Pixels> {
        let bounds = self.0.get();
        point(bounds.left(), bounds.bottom() + px(MENU_GAP))
    }

    pub(crate) fn wrap(&self, trigger: impl IntoElement) -> Div {
        let cell = self.0.clone();
        div()
            .on_children_prepainted(move |bounds, _, _| {
                if let Some(first) = bounds.first() {
                    cell.set(*first);
                }
            })
            .child(trigger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn the_menu_opens_just_below_the_left_edge_of_its_trigger() {
        let anchor = MenuAnchor::default();
        anchor
            .0
            .set(Bounds::new(point(px(30.), px(50.)), size(px(80.), px(28.))));
        assert_eq!(anchor.below(), point(px(30.), px(82.)));
    }
}
