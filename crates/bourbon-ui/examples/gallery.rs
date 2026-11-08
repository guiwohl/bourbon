use gpui::{
    AnyView, App, AppContext as _, Bounds, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_platform::application;

use bourbon_ui::palette::solid;
use bourbon_ui::{
    Appearance, Button, ButtonVariant, Divider, Icon, IconAssets, IconButton, IconButtonSize,
    IconName, ListRow, Menu, MenuEntry, MenuItem, Pill, ScrollArea, SectionHeader, Segment,
    SegmentedControl, Splitter, Theme, ThemeExt as _, Themed, Tooltip, register_fonts,
};

const APP_ID: &str = "bourbon-gallery";
const TITLE: &str = "Bourbon Gallery";
const PILL_COLOURS: [(&str, u32); 6] = [
    ("Design", 0x003b_82f6),
    ("Urgent", 0x00ef_4444),
    ("Ideas", 0x00ea_b308),
    ("Home", 0x0022_c55e),
    ("Reading", 0x008b_5cf6),
    ("Archive", 0x006b_7280),
];

struct Gallery {
    menus: [Entity<Menu>; 2],
    tooltips: [AnyView; 2],
    scrolls: [ScrollHandle; 2],
}

fn demo_menu(cx: &mut Context<Gallery>) -> Entity<Menu> {
    cx.new(|cx| {
        Menu::new(
            vec![
                MenuEntry::Item(
                    MenuItem::new("Move to Notebook")
                        .icon(IconName::Notebook)
                        .submenu(),
                ),
                MenuEntry::Item(
                    MenuItem::new("Duplicate")
                        .icon(IconName::Copy)
                        .shortcut("Ctrl+D"),
                ),
                MenuEntry::Item(
                    MenuItem::new("Pin to Top")
                        .icon(IconName::Pin)
                        .checked(true),
                ),
                MenuEntry::Separator,
                MenuEntry::Item(MenuItem::new("Revision History").disabled(true)),
                MenuEntry::Item(MenuItem::new("Move to Trash").icon(IconName::Trash)),
            ],
            cx,
        )
    })
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            menus: [demo_menu(cx), demo_menu(cx)],
            tooltips: [
                Tooltip::new("Toggle sidebar", Some(SharedString::from("Ctrl+/")), cx).into(),
                Tooltip::new("Toggle sidebar", Some(SharedString::from("Ctrl+/")), cx).into(),
            ],
            scrolls: [ScrollHandle::new(), ScrollHandle::new()],
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().size_full().children(
            [Appearance::Light, Appearance::Dark]
                .into_iter()
                .enumerate()
                .map(|(index, appearance)| {
                    Themed::new(
                        appearance,
                        Column {
                            menu: self.menus[index].clone(),
                            tooltip: self.tooltips[index].clone(),
                            scroll: self.scrolls[index].clone(),
                            index,
                        },
                    )
                }),
        )
    }
}

#[derive(IntoElement)]
struct Column {
    menu: Entity<Menu>,
    tooltip: AnyView,
    scroll: ScrollHandle,
    index: usize,
}

fn section(title: &'static str, cx: &App) -> gpui::Div {
    let tokens = &cx.theme().tokens;
    div().flex().flex_col().gap(px(8.)).child(
        div()
            .text_size(tokens.scale.tiny)
            .text_color(tokens.colors.unselected_text)
            .child(title),
    )
}

fn buttons(index: usize, cx: &App) -> gpui::Div {
    section("Buttons", cx).child(
        div()
            .flex()
            .gap(px(8.))
            .child(
                Button::new(("plain", index))
                    .label("Plain")
                    .variant(ButtonVariant::Plain),
            )
            .child(
                Button::new(("subtle", index))
                    .label("Subtle")
                    .icon(IconName::Plus),
            )
            .child(
                Button::new(("primary", index))
                    .label("Primary")
                    .variant(ButtonVariant::Primary),
            )
            .child(
                Button::new(("disabled", index))
                    .label("Disabled")
                    .disabled(true),
            ),
    )
}

fn icon_buttons(index: usize, tooltip: AnyView, cx: &App) -> gpui::Div {
    section("Icon buttons and tooltip", cx).child(
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .child(IconButton::new(("back", index), IconName::ArrowLeft).tooltip("Back"))
            .child(IconButton::new(("forward", index), IconName::ArrowRight).disabled(true))
            .child(IconButton::new(("sidebar", index), IconName::PanelLeft).selected(true))
            .child(IconButton::new(("more", index), IconName::EllipsisVertical))
            .child(
                IconButton::new(("search", index), IconName::Search)
                    .size(IconButtonSize::Accessory),
            )
            .child(tooltip),
    )
}

fn view_mode(index: usize, cx: &App) -> gpui::Div {
    section("Segmented control (view mode)", cx).child(
        SegmentedControl::new(
            SharedString::from(format!("view-mode-{index}")),
            vec![
                Segment::new(IconName::Pencil, "Show Editor"),
                Segment::new(IconName::Columns2, "View Side by Side"),
                Segment::new(IconName::Eye, "Show Preview"),
            ],
        )
        .selected(1),
    )
}

fn pills(cx: &App) -> gpui::Div {
    section("Pills", cx).child(
        div().flex().flex_wrap().gap(px(6.)).children(
            PILL_COLOURS
                .iter()
                .map(|(label, hex)| Pill::new(*label, solid(*hex))),
        ),
    )
}

fn sidebar_rows(index: usize, cx: &App) -> gpui::Div {
    let tokens = &cx.theme().tokens;
    section("Sidebar rows", cx).child(
        div()
            .w(tokens.scale.sidebar_width.default)
            .bg(tokens.colors.sidebar_background)
            .py(px(4.))
            .child(
                ListRow::new(("all", index), "All Notes")
                    .icon(IconName::FileText)
                    .count(42),
            )
            .child(
                ListRow::new(("inbox", index), "Inbox")
                    .icon(IconName::Inbox)
                    .count(7)
                    .selected(true)
                    .pane_focused(true),
            )
            .child(
                ListRow::new(("work", index), "Work")
                    .icon(IconName::Briefcase)
                    .disclosure(true)
                    .selected(true),
            )
            .child(
                ListRow::new(("sub", index), "Projects")
                    .icon(IconName::Notebook)
                    .depth(1)
                    .count(3),
            )
            .child(SectionHeader::new(("tags", index), "Tags").collapsible(false, |_, _, _| {}))
            .child(ListRow::new(("trash", index), "Trash").icon(IconName::Trash)),
    )
}

fn splitter(index: usize, cx: &App) -> gpui::Div {
    let tokens = &cx.theme().tokens;
    let sidebar = tokens.scale.sidebar_width;
    section("Splitter and divider", cx)
        .child(
            div()
                .flex()
                .h(px(60.))
                .child(
                    div()
                        .w(sidebar.default)
                        .bg(tokens.colors.sidebar_background),
                )
                .child(Splitter::new(("splitter", index), sidebar.default, sidebar))
                .child(div().flex_1().bg(tokens.colors.note_list_background)),
        )
        .child(Divider::horizontal())
}

fn scroll_area(index: usize, scroll: &ScrollHandle, cx: &App) -> gpui::Div {
    section("Scroll area", cx).child(
        div().h(px(90.)).w(px(260.)).child(
            ScrollArea::new(("scroll", index), scroll)
                .children((1..=20).map(|row| div().h(px(24.)).child(format!("Row {row}")))),
        ),
    )
}

fn icons(cx: &App) -> gpui::Div {
    section("Icons", cx).child(
        div().flex().flex_wrap().gap(px(10.)).children(
            IconName::ALL
                .iter()
                .map(|name| Icon::new(*name).size(px(18.))),
        ),
    )
}

impl RenderOnce for Column {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = cx.theme().tokens.clone();
        let colors = tokens.colors;
        let index = self.index;
        div()
            .id(("column", index))
            .flex()
            .flex_col()
            .gap(px(18.))
            .flex_1()
            .h_full()
            .p(px(20.))
            .overflow_y_scroll()
            .bg(colors.page_background)
            .text_color(colors.text)
            .font_family(tokens.fonts.ui_family.clone())
            .text_size(tokens.scale.medium)
            .child(buttons(index, cx))
            .child(icon_buttons(index, self.tooltip, cx))
            .child(view_mode(index, cx))
            .child(pills(cx))
            .child(sidebar_rows(index, cx))
            .child(section("Menu", cx).child(self.menu))
            .child(splitter(index, cx))
            .child(scroll_area(index, &self.scroll, cx))
            .child(icons(cx))
    }
}

fn main() {
    application().with_assets(IconAssets).run(|cx: &mut App| {
        register_fonts(cx);
        bourbon_ui::init(cx);
        Theme::install(Appearance::Light, cx);
        let bounds = Bounds::centered(None, size(px(1280.), px(980.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from(TITLE)),
                ..TitlebarOptions::default()
            }),
            app_id: Some(APP_ID.to_owned()),
            ..WindowOptions::default()
        };
        match cx.open_window(options, |_, cx| cx.new(Gallery::new)) {
            Ok(_) => cx.activate(true),
            Err(error) => {
                eprintln!("bourbon-gallery: cannot open the window: {error}");
                cx.quit();
            }
        }
    });
}
