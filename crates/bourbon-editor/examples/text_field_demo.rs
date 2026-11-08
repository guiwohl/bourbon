use bourbon_editor::{TextField, bind_text_field_keys};
use bourbon_ui::{
    Appearance, IconAssets, IconName, Theme, ThemeExt as _, Themed, Tokens, register_fonts,
};
use gpui::{
    App, Bounds, Context, Entity, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;

const COLUMN_WIDTH: f32 = 320.;

struct Fields {
    filter: Entity<TextField>,
    empty_filter: Entity<TextField>,
    rename: Entity<TextField>,
}

impl Fields {
    fn new(cx: &mut App) -> Self {
        Self {
            filter: cx.new(|cx| {
                let mut field = TextField::new(cx)
                    .placeholder("Filter")
                    .leading_icon(IconName::Search);
                field.set_text("budget", cx);
                field
            }),
            empty_filter: cx.new(|cx| {
                TextField::new(cx)
                    .placeholder("Filter")
                    .leading_icon(IconName::Search)
            }),
            rename: cx.new(|cx| {
                let mut field = TextField::new(cx);
                field.set_text("Work in progress", cx);
                field
            }),
        }
    }
}

struct Demo {
    light: Fields,
    dark: Fields,
}

fn column(appearance: Appearance, fields: &Fields) -> impl IntoElement {
    let tokens = Tokens::for_appearance(appearance);
    let section = |label: &'static str, field: &Entity<TextField>| {
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .text_size(tokens.scale.tiny)
                    .text_color(tokens.colors.muted_text)
                    .child(SharedString::from(label)),
            )
            .child(field.clone())
    };
    Themed::new(
        appearance,
        div()
            .w(px(COLUMN_WIDTH))
            .h_full()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .font_family(tokens.fonts.ui_family.clone())
            .bg(tokens.colors.note_list_background)
            .child(section("Filter", &fields.filter))
            .child(section("Filter, empty", &fields.empty_filter))
            .child(
                div()
                    .p(px(8.))
                    .bg(tokens.colors.input_background)
                    .child(section("Rename notebook", &fields.rename)),
            ),
    )
}

impl Render for Demo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(cx.theme().tokens.colors.page_background)
            .child(column(Appearance::Light, &self.light))
            .child(column(Appearance::Dark, &self.dark))
    }
}

fn main() {
    application().with_assets(IconAssets).run(|cx: &mut App| {
        register_fonts(cx);
        bind_text_field_keys(cx);
        Theme::install(Appearance::Light, cx);
        let bounds = Bounds::centered(None, size(px(2. * COLUMN_WIDTH), px(300.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            app_id: Some("bourbon-text-field-demo".to_owned()),
            titlebar: Some(TitlebarOptions {
                title: Some("Bourbon Text Field Demo".into()),
                ..TitlebarOptions::default()
            }),
            ..WindowOptions::default()
        };
        let opened = cx.open_window(options, |window, cx| {
            let demo = Demo {
                light: Fields::new(cx),
                dark: Fields::new(cx),
            };
            demo.light
                .filter
                .update(cx, |field, cx| field.focus(window, cx));
            cx.new(|_| demo)
        });
        match opened {
            Ok(_) => cx.activate(true),
            Err(error) => {
                eprintln!("text field demo: cannot open the window: {error}");
                cx.quit();
            }
        }
    });
}
