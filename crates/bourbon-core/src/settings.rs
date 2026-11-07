use std::io::{self, Write as _};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

pub const SIDEBAR_WIDTH_RANGE: RangeInclusive<u16> = 160..=400;
pub const NOTE_LIST_WIDTH_RANGE: RangeInclusive<u16> = 220..=560;

const DEFAULT_SIDEBAR_WIDTH: u16 = 200;
const DEFAULT_NOTE_LIST_WIDTH: u16 = 260;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    #[default]
    Bourbon,
    Omarchy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    #[default]
    Edit,
    SideBySide,
    Preview,
}

impl ViewMode {
    pub const ALL: [Self; 3] = [Self::Edit, Self::SideBySide, Self::Preview];

    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Edit => Self::SideBySide,
            Self::SideBySide => Self::Preview,
            Self::Preview => Self::Edit,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Edit => "Edit",
            Self::SideBySide => "Side-by-side",
            Self::Preview => "Preview",
        }
    }

    #[must_use]
    pub fn shows_editor(self) -> bool {
        self != Self::Preview
    }

    #[must_use]
    pub fn shows_preview(self) -> bool {
        self != Self::Edit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupAuth {
    #[default]
    GhCli,
    Token,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Settings {
    pub theme_source: ThemeSource,
    pub appearance: Appearance,
    pub sidebar_visible: bool,
    pub backup_repository: Option<String>,
    pub backup_auth: BackupAuth,
    pub backup_enabled: bool,
    pub editor_view_mode: ViewMode,
    sidebar_width: u16,
    note_list_width: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_source: ThemeSource::default(),
            appearance: Appearance::default(),
            sidebar_visible: true,
            backup_repository: None,
            backup_auth: BackupAuth::default(),
            backup_enabled: false,
            editor_view_mode: ViewMode::default(),
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
            note_list_width: DEFAULT_NOTE_LIST_WIDTH,
        }
    }
}

impl Settings {
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let Ok(table) = text.parse::<toml::Table>() else {
            return Self::default();
        };
        let defaults = Self::default();
        let mut settings = Self {
            theme_source: field(&table, "theme_source").unwrap_or(defaults.theme_source),
            appearance: field(&table, "appearance").unwrap_or(defaults.appearance),
            sidebar_visible: field(&table, "sidebar_visible").unwrap_or(defaults.sidebar_visible),
            backup_repository: field(&table, "backup_repository"),
            backup_auth: field(&table, "backup_auth").unwrap_or(defaults.backup_auth),
            backup_enabled: field(&table, "backup_enabled").unwrap_or(defaults.backup_enabled),
            editor_view_mode: field(&table, "editor_view_mode")
                .unwrap_or(defaults.editor_view_mode),
            ..defaults
        };
        if let Some(width) = field(&table, "sidebar_width") {
            settings.set_sidebar_width(width);
        }
        if let Some(width) = field(&table, "note_list_width") {
            settings.set_note_list_width(width);
        }
        settings
    }

    #[must_use]
    pub fn to_toml(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }

    #[must_use]
    pub fn sidebar_width(&self) -> u16 {
        self.sidebar_width
    }

    #[must_use]
    pub fn note_list_width(&self) -> u16 {
        self.note_list_width
    }

    pub fn set_sidebar_width(&mut self, width: u16) {
        self.sidebar_width = clamp(width, &SIDEBAR_WIDTH_RANGE);
    }

    pub fn set_note_list_width(&mut self, width: u16) {
        self.note_list_width = clamp(width, &NOTE_LIST_WIDTH_RANGE);
    }
}

fn clamp(width: u16, range: &RangeInclusive<u16>) -> u16 {
    width.clamp(*range.start(), *range.end())
}

fn field<T: serde::de::DeserializeOwned>(table: &toml::Table, key: &str) -> Option<T> {
    table.get(key)?.clone().try_into().ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsFile {
    path: PathBuf,
}

impl SettingsFile {
    #[must_use]
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn default_location() -> Self {
        Self::at(default_settings_path())
    }

    #[must_use]
    pub fn load(&self) -> Settings {
        std::fs::read_to_string(&self.path)
            .map(|text| Settings::parse(&text))
            .unwrap_or_default()
    }

    pub fn save(&self, settings: &Settings) -> io::Result<()> {
        let directory = self.path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(directory)?;
        let mut staged = NamedTempFile::new_in(directory)?;
        staged.write_all(settings.to_toml().as_bytes())?;
        staged.persist(&self.path).map_err(|error| error.error)?;
        Ok(())
    }
}

#[must_use]
pub fn default_settings_path() -> PathBuf {
    resolve_settings_path(
        std::env::var("XDG_CONFIG_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
}

fn resolve_settings_path(xdg_config_home: Option<&str>, home: Option<&str>) -> PathBuf {
    let config_directory = match (xdg_config_home, home) {
        (Some(xdg), _) if !xdg.is_empty() => PathBuf::from(xdg),
        (_, Some(home)) => PathBuf::from(home).join(".config"),
        _ => PathBuf::from(".config"),
    };
    config_directory.join("bourbon").join("settings.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_config_home_wins() {
        assert_eq!(
            resolve_settings_path(Some("/x/config"), Some("/home/u")),
            PathBuf::from("/x/config/bourbon/settings.toml")
        );
    }

    #[test]
    fn falls_back_to_home_dot_config() {
        assert_eq!(
            resolve_settings_path(None, Some("/home/u")),
            PathBuf::from("/home/u/.config/bourbon/settings.toml")
        );
    }

    #[test]
    fn empty_xdg_config_home_is_ignored() {
        assert_eq!(
            resolve_settings_path(Some(""), Some("/home/u")),
            PathBuf::from("/home/u/.config/bourbon/settings.toml")
        );
    }

    #[test]
    fn without_any_environment_the_path_is_relative() {
        assert_eq!(
            resolve_settings_path(None, None),
            PathBuf::from(".config/bourbon/settings.toml")
        );
    }

    #[test]
    fn default_path_ends_with_settings_toml() {
        assert!(default_settings_path().ends_with("bourbon/settings.toml"));
    }

    #[test]
    fn a_missing_file_loads_the_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let file = SettingsFile::at(directory.path().join("settings.toml"));
        assert_eq!(file.load(), Settings::default());
    }

    #[test]
    fn defaults_are_bourbon_light_with_visible_sidebar() {
        let settings = Settings::default();
        assert_eq!(settings.theme_source, ThemeSource::Bourbon);
        assert_eq!(settings.appearance, Appearance::Light);
        assert!(settings.sidebar_visible);
        assert_eq!(settings.sidebar_width(), 200);
        assert_eq!(settings.note_list_width(), 260);
    }

    #[test]
    fn saved_settings_load_back_identically() {
        let directory = tempfile::tempdir().unwrap();
        let file = SettingsFile::at(directory.path().join("nested").join("settings.toml"));
        let mut settings = Settings {
            theme_source: ThemeSource::Omarchy,
            appearance: Appearance::Light,
            sidebar_visible: false,
            ..Settings::default()
        };
        settings.set_sidebar_width(300);
        settings.set_note_list_width(400);

        file.save(&settings).unwrap();

        assert_eq!(file.load(), settings);
    }

    #[test]
    fn saving_replaces_the_previous_file_and_leaves_no_temporary_files() {
        let directory = tempfile::tempdir().unwrap();
        let file = SettingsFile::at(directory.path().join("settings.toml"));
        file.save(&Settings::default()).unwrap();
        let second = Settings {
            sidebar_visible: false,
            ..Settings::default()
        };

        file.save(&second).unwrap();

        assert_eq!(file.load(), second);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn unparsable_text_loads_the_defaults() {
        assert_eq!(Settings::parse("this is = = not toml"), Settings::default());
    }

    #[test]
    fn invalid_fields_fall_back_individually_and_valid_ones_survive() {
        let settings = Settings::parse(
            "theme_source = \"omarchy\"\nappearance = \"purple\"\nsidebar_visible = \"yes\"\nsidebar_width = \"wide\"\nunknown = 1\n",
        );
        assert_eq!(settings.theme_source, ThemeSource::Omarchy);
        assert_eq!(settings.appearance, Appearance::Light);
        assert!(settings.sidebar_visible);
        assert_eq!(settings.sidebar_width(), 200);
    }

    #[test]
    fn widths_are_clamped_when_loaded() {
        let settings = Settings::parse("sidebar_width = 5\nnote_list_width = 9000\n");
        assert_eq!(settings.sidebar_width(), 160);
        assert_eq!(settings.note_list_width(), 560);
    }

    #[test]
    fn widths_are_clamped_when_set() {
        let mut settings = Settings::default();
        settings.set_sidebar_width(1000);
        settings.set_note_list_width(10);
        assert_eq!(settings.sidebar_width(), 400);
        assert_eq!(settings.note_list_width(), 220);
        settings.set_sidebar_width(200);
        settings.set_note_list_width(300);
        assert_eq!(settings.sidebar_width(), 200);
        assert_eq!(settings.note_list_width(), 300);
    }

    #[test]
    fn light_appearance_and_hidden_sidebar_parse() {
        let settings = Settings::parse("appearance = \"light\"\nsidebar_visible = false\n");
        assert_eq!(settings.appearance, Appearance::Light);
        assert!(!settings.sidebar_visible);
    }

    #[test]
    fn backup_defaults_are_off_with_the_gh_cli() {
        let settings = Settings::default();
        assert_eq!(settings.backup_repository, None);
        assert_eq!(settings.backup_auth, BackupAuth::GhCli);
        assert!(!settings.backup_enabled);
    }

    #[test]
    fn backup_fields_round_trip_through_the_file() {
        let directory = tempfile::tempdir().unwrap();
        let file = SettingsFile::at(directory.path().join("settings.toml"));
        let settings = Settings {
            backup_repository: Some("owner/notes".to_owned()),
            backup_auth: BackupAuth::Token,
            backup_enabled: true,
            ..Settings::default()
        };

        file.save(&settings).unwrap();

        assert_eq!(file.load(), settings);
        assert!(
            std::fs::read_to_string(directory.path().join("settings.toml"))
                .unwrap()
                .contains("backup_auth = \"token\"")
        );
    }

    #[test]
    fn invalid_backup_fields_fall_back_individually() {
        let settings = Settings::parse(
            "backup_repository = 5\nbackup_auth = \"password\"\nbackup_enabled = true\n",
        );
        assert_eq!(settings.backup_repository, None);
        assert_eq!(settings.backup_auth, BackupAuth::GhCli);
        assert!(settings.backup_enabled);
    }

    #[test]
    fn the_view_mode_defaults_to_edit_and_round_trips() {
        assert_eq!(Settings::default().editor_view_mode, ViewMode::Edit);
        let directory = tempfile::tempdir().unwrap();
        let file = SettingsFile::at(directory.path().join("settings.toml"));
        let settings = Settings {
            editor_view_mode: ViewMode::SideBySide,
            ..Settings::default()
        };

        file.save(&settings).unwrap();

        assert_eq!(file.load(), settings);
        assert!(
            std::fs::read_to_string(directory.path().join("settings.toml"))
                .unwrap()
                .contains("editor_view_mode = \"side_by_side\"")
        );
    }

    #[test]
    fn an_invalid_view_mode_falls_back_alone() {
        let settings = Settings::parse("editor_view_mode = \"split\"\nbackup_enabled = true\n");
        assert_eq!(settings.editor_view_mode, ViewMode::Edit);
        assert!(settings.backup_enabled);
    }

    #[test]
    fn cycling_visits_edit_side_by_side_preview_and_wraps() {
        assert_eq!(ViewMode::Edit.next(), ViewMode::SideBySide);
        assert_eq!(ViewMode::SideBySide.next(), ViewMode::Preview);
        assert_eq!(ViewMode::Preview.next(), ViewMode::Edit);
    }

    #[test]
    fn each_mode_shows_the_panes_it_names() {
        assert!(ViewMode::Edit.shows_editor() && !ViewMode::Edit.shows_preview());
        assert!(ViewMode::SideBySide.shows_editor() && ViewMode::SideBySide.shows_preview());
        assert!(!ViewMode::Preview.shows_editor() && ViewMode::Preview.shows_preview());
    }

    #[test]
    fn labels_name_the_modes() {
        let labels = ViewMode::ALL.map(ViewMode::label);
        assert_eq!(labels, ["Edit", "Side-by-side", "Preview"]);
    }
}
