use std::rc::Rc;

use bourbon_ui::{Menu, MenuEntry, MenuEvent, MenuItem, Popover};
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, IntoElement, Pixels, Point, Subscription, Window,
};

use super::Workspace;

type MenuAction = Rc<dyn Fn(&mut Workspace, &mut Window, &mut Context<Workspace>)>;

#[derive(Default)]
pub(crate) struct MenuBuilder {
    entries: Vec<MenuEntry>,
    actions: Vec<Option<MenuAction>>,
}

impl MenuBuilder {
    pub(crate) fn item(
        mut self,
        item: MenuItem,
        action: impl Fn(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static,
    ) -> Self {
        self.entries.push(MenuEntry::Item(item));
        self.actions.push(Some(Rc::new(action)));
        self
    }

    #[cfg(test)]
    pub(crate) fn item_labels(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                MenuEntry::Item(item) => Some(item.label.to_string()),
                MenuEntry::Separator => None,
            })
            .collect()
    }

    pub(crate) fn separator(mut self) -> Self {
        self.entries.push(MenuEntry::Separator);
        self.actions.push(None);
        self
    }
}

pub(crate) struct OpenMenu {
    position: Point<Pixels>,
    menu: Entity<Menu>,
    restore_focus: Option<FocusHandle>,
    _subscription: Subscription,
}

impl OpenMenu {
    #[cfg(test)]
    pub(crate) fn menu(&self) -> &Entity<Menu> {
        &self.menu
    }
}

impl Workspace {
    pub(crate) fn open_menu(
        &mut self,
        position: Point<Pixels>,
        builder: MenuBuilder,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let restore_focus = match self.open_menu.take() {
            Some(previous) => previous.restore_focus,
            None => window.focused(cx),
        };
        self.menu_origin = position;
        let MenuBuilder { entries, actions } = builder;
        let menu = cx.new(|cx| Menu::new(entries, cx));
        let subscription = cx.subscribe_in(
            &menu,
            window,
            move |workspace, _, event: &MenuEvent, window, cx| match event {
                MenuEvent::Confirmed(index) => {
                    workspace.close_menu(window, cx);
                    if let Some(Some(action)) = actions.get(*index) {
                        action(workspace, window, cx);
                    }
                }
                MenuEvent::Dismissed => workspace.close_menu(window, cx),
            },
        );
        let focusing = menu.clone();
        window.defer(cx, move |window, cx| {
            focusing.update(cx, |menu, cx| menu.focus(window, cx));
        });
        self.open_menu = Some(OpenMenu {
            position,
            menu,
            restore_focus,
            _subscription: subscription,
        });
        cx.notify();
    }

    pub(crate) fn last_menu_position(&self) -> Point<Pixels> {
        self.menu_origin
    }

    pub(crate) fn close_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(closed) = self.open_menu.take() {
            if let Some(previous) = closed.restore_focus {
                window.focus(&previous, cx);
            }
            cx.notify();
        }
    }

    pub(super) fn menu_popover(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        let open = self.open_menu.as_ref()?;
        let view = cx.entity();
        Some(
            Popover::at(open.position, open.menu.clone()).on_dismiss(move |window, cx| {
                view.update(cx, |workspace, cx| workspace.close_menu(window, cx));
            }),
        )
    }
}
