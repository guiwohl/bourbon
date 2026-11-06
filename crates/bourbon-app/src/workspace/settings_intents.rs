use bourbon_core::{Appearance, BackupAuth, GitHubRepository, ThemeSource};
use gpui_kit::Context;

use super::Workspace;
use crate::theme;

impl Workspace {
    pub(crate) fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_open = true;
        cx.notify();
    }

    pub(crate) fn close_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_open = false;
        cx.notify();
    }

    pub(crate) fn set_theme_source(&mut self, source: ThemeSource, cx: &mut Context<Self>) {
        self.settings.theme_source = source;
        self.apply_theme_and_persist(cx);
    }

    pub(crate) fn set_appearance(&mut self, appearance: Appearance, cx: &mut Context<Self>) {
        self.settings.appearance = appearance;
        self.apply_theme_and_persist(cx);
    }

    pub(crate) fn set_backup_auth(&mut self, auth: BackupAuth, cx: &mut Context<Self>) {
        self.settings.backup_auth = auth;
        self.persist_settings(cx);
        cx.notify();
    }

    pub(crate) fn toggle_backup_enabled(&mut self, cx: &mut Context<Self>) {
        self.settings.backup_enabled = !self.settings.backup_enabled;
        self.persist_settings(cx);
        cx.notify();
    }

    pub(super) fn repository_changed(&mut self, cx: &mut Context<Self>) {
        let text = self.repository_input.read(cx).value().trim().to_owned();
        if text.is_empty() {
            self.settings.backup_repository = None;
            self.data.repository_error = None;
        } else if let Err(error) = GitHubRepository::parse(&text) {
            self.data.repository_error = Some(error.to_string().into());
            cx.notify();
            return;
        } else {
            self.settings.backup_repository = Some(text);
            self.data.repository_error = None;
        }
        self.persist_settings(cx);
        cx.notify();
    }

    pub(super) fn token_changed(&mut self, cx: &mut Context<Self>) {
        let token = self.token_input.read(cx).unmask_value();
        if let Err(error) = self.locations.backup_token.save(&token) {
            self.error_message = Some(format!("cannot save the token: {error}").into());
            cx.notify();
        }
    }

    pub(crate) fn resize_sidebar(&mut self, width: f32, cx: &mut Context<Self>) {
        self.settings.set_sidebar_width(rounded_width(width));
        self.widths_changed = true;
        cx.notify();
    }

    pub(crate) fn resize_note_list(&mut self, width: f32, cx: &mut Context<Self>) {
        self.settings.set_note_list_width(rounded_width(width));
        self.widths_changed = true;
        cx.notify();
    }

    pub(crate) fn resize_note_list_to_pointer(&mut self, pointer: f32, cx: &mut Context<Self>) {
        let sidebar_edge = if self.sidebar.is_visible {
            f32::from(self.settings.sidebar_width())
        } else {
            0.
        };
        self.resize_note_list(pointer - sidebar_edge, cx);
    }

    pub(crate) fn finish_resizing(&mut self, cx: &mut Context<Self>) {
        if std::mem::take(&mut self.widths_changed) {
            self.persist_settings(cx);
        }
    }

    pub(super) fn reload_omarchy_theme_if_changed(&mut self, cx: &mut Context<Self>) {
        if self.omarchy_watch.has_changed() && self.settings.theme_source == ThemeSource::Omarchy {
            theme::apply(&self.settings, &self.locations.omarchy_colors, cx);
        }
    }

    pub(super) fn persist_settings(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.locations.settings.save(&self.settings) {
            self.error_message = Some(format!("cannot save the settings: {error}").into());
            cx.notify();
        }
    }

    fn apply_theme_and_persist(&mut self, cx: &mut Context<Self>) {
        theme::apply(&self.settings, &self.locations.omarchy_colors, cx);
        self.persist_settings(cx);
        cx.notify();
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rounded_width(width: f32) -> u16 {
    width.round().clamp(0., f32::from(u16::MAX)) as u16
}
