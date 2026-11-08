pub mod button;
pub mod checkbox;
pub mod dialog;
pub mod divider;
pub mod fonts;
pub mod icon;
pub mod list_row;
pub mod menu;
pub mod modal;
pub mod navigation;
pub mod palette;
pub mod pill;
pub mod popover;
pub mod readable;
pub mod scroll_area;
pub mod segmented;
pub mod select;
pub mod splitter;
pub mod theme;
pub mod tokens;
pub mod tooltip;

#[cfg(test)]
mod test_support;

pub use button::{Button, ButtonVariant, IconButton, IconButtonSize};
pub use checkbox::Checkbox;
pub use dialog::Dialog;
pub use divider::Divider;
pub use fonts::{UI_FAMILY, register_fonts};
pub use icon::{Icon, IconAssets, IconName, Stroke};
pub use list_row::{ListRow, SectionHeader};
pub use menu::{Menu, MenuEntry, MenuEvent, MenuItem};
pub use modal::{Modal, Placement};
pub use pill::Pill;
pub use popover::Popover;
pub use readable::pill_text_color;
pub use scroll_area::ScrollArea;
pub use segmented::{Segment, SegmentedControl};
pub use select::Select;
pub use splitter::{Splitter, WidthLimits};
pub use theme::{Appearance, Theme, ThemeExt, Themed};
pub use tokens::{Colors, Fonts, Scale, Shadows, Tokens};
pub use tooltip::Tooltip;

pub fn init(cx: &mut gpui::App) {
    menu::bind_keys(cx);
    modal::bind_keys(cx);
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;

    #[gpui::test]
    fn init_binds_the_menu_and_modal_keys(cx: &mut TestAppContext) {
        let before = cx.update(|cx| cx.key_bindings().borrow().bindings().count());

        cx.update(super::init);

        let after = cx.update(|cx| cx.key_bindings().borrow().bindings().count());
        assert_eq!(after - before, 8);
    }
}
