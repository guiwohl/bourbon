use std::borrow::Cow;

use bourbon_ui::IconAssets;
use gpui_kit::assets::Assets;
use gpui_kit::{AssetSource, Result, SharedString};

pub(crate) struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match IconAssets.load(path)? {
            Some(icon) => Ok(Some(icon)),
            None => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut listed = IconAssets.list(path)?;
        listed.extend(Assets.list(path)?);
        Ok(listed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kit_icons_are_served_from_bourbon_ui_first() {
        let ours = IconAssets.load("icons/notebook.svg").unwrap().unwrap();
        let served = AppAssets.load("icons/notebook.svg").unwrap().unwrap();
        assert_eq!(ours, served);
    }

    #[test]
    fn assets_unknown_to_bourbon_ui_fall_back_to_the_component_assets() {
        assert!(IconAssets.load("icons/arrow-up.svg").unwrap().is_none());
        assert!(AppAssets.load("icons/arrow-up.svg").unwrap().is_some());
    }

    #[test]
    fn unknown_paths_are_not_found() {
        assert!(!matches!(
            AppAssets.load("icons/nope-nope.svg"),
            Ok(Some(_))
        ));
    }

    #[test]
    fn listing_the_icon_directory_includes_bourbon_ui_icons() {
        let listed = AppAssets.list("icons").unwrap();
        assert!(
            listed
                .iter()
                .any(|name| name.as_ref() == "icons/notebook.svg")
        );
    }
}
