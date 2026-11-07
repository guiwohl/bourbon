use gpui::{
    AnyElement, App, Context, IntoElement, Render, TestAppContext, VisualTestContext, Window,
};

use crate::menu;
use crate::theme::{Appearance, Theme};

pub(crate) struct Host {
    build: Box<dyn Fn() -> AnyElement>,
}

impl Host {
    pub(crate) fn new(build: impl Fn() -> AnyElement + 'static) -> Self {
        Self {
            build: Box::new(build),
        }
    }
}

impl Render for Host {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        (self.build)()
    }
}

pub(crate) fn prepare(cx: &mut TestAppContext) {
    cx.update(|cx: &mut App| {
        Theme::install(Appearance::Light, cx);
        menu::bind_keys(cx);
    });
}

pub(crate) fn host(
    cx: &mut TestAppContext,
    build: impl Fn() -> AnyElement + 'static,
) -> &mut VisualTestContext {
    prepare(cx);
    let (_, visual) = cx.add_window_view(|_, _| Host::new(build));
    visual.run_until_parked();
    visual
}
