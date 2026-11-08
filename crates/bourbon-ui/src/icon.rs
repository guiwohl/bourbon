use std::borrow::Cow;

use gpui::{
    App, AssetSource, Hsla, IntoElement, Pixels, RenderOnce, Result, SharedString, Styled as _,
    Transformation, Window, px, radians, svg,
};

pub const REGULAR_STROKE: &str = "1.5";
const AUTHORED_STROKE: &str = "stroke-width=\"2\"";
const ICON_DIRECTORY: &str = "icons";

macro_rules! icons {
    ($($variant:ident => $file:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum IconName {
            $($variant),*
        }

        impl IconName {
            pub const ALL: &'static [IconName] = &[$(IconName::$variant),*];

            #[must_use]
            pub const fn file_name(self) -> &'static str {
                match self {
                    $(IconName::$variant => $file),*
                }
            }
        }

        const SOURCES: &[(&str, &str)] = &[
            $(($file, include_str!(concat!("../assets/icons/", $file, ".svg")))),*
        ];
    };
}

icons! {
    ArrowLeft => "arrow-left",
    ArrowRight => "arrow-right",
    Briefcase => "briefcase",
    Check => "check",
    ChevronDown => "chevron-down",
    ChevronRight => "chevron-right",
    ChevronUp => "chevron-up",
    Circle => "circle",
    CircleCheck => "circle-check",
    CirclePause => "circle-pause",
    CirclePlay => "circle-play",
    CircleX => "circle-x",
    Columns2 => "columns-2",
    Copy => "copy",
    EllipsisVertical => "ellipsis-vertical",
    Eye => "eye",
    FileText => "file-text",
    Funnel => "funnel",
    Inbox => "inbox",
    ListChecks => "list-checks",
    Maximize => "maximize",
    Notebook => "notebook",
    PanelLeft => "panel-left",
    Pencil => "pencil",
    Pin => "pin",
    Plus => "plus",
    Search => "search",
    Settings => "settings",
    SquarePen => "square-pen",
    Tag => "tag",
    Trash => "trash",
    X => "x",
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stroke {
    Regular,
    Medium,
    Heavy,
}

impl Stroke {
    const fn width(self) -> &'static str {
        match self {
            Stroke::Regular => REGULAR_STROKE,
            Stroke::Medium => "2.5",
            Stroke::Heavy => "3",
        }
    }
}

#[must_use]
pub fn icon_path(name: IconName, stroke: Stroke) -> String {
    match stroke {
        Stroke::Regular => format!("{ICON_DIRECTORY}/{}.svg", name.file_name()),
        other => format!(
            "{ICON_DIRECTORY}/{}@{}.svg",
            name.file_name(),
            other.width()
        ),
    }
}

fn parse_request(path: &str) -> Option<(&'static str, &str)> {
    let file = path.strip_prefix(ICON_DIRECTORY)?.strip_prefix('/')?;
    let stem = file.strip_suffix(".svg")?;
    let (name, stroke) = stem.split_once('@').unwrap_or((stem, REGULAR_STROKE));
    let valid_stroke = stroke
        .parse::<f32>()
        .is_ok_and(|width| width.is_finite() && width > 0. && width <= 4.)
        && stroke
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.');
    let (_, source) = SOURCES.iter().find(|(file_name, _)| *file_name == name)?;
    valid_stroke.then_some((*source, stroke))
}

pub struct IconAssets;

impl AssetSource for IconAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(parse_request(path).map(|(source, stroke)| {
            let restroked = source.replace(AUTHORED_STROKE, &format!("stroke-width=\"{stroke}\""));
            Cow::Owned(restroked.into_bytes())
        }))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path.trim_end_matches('/') != ICON_DIRECTORY {
            return Ok(Vec::new());
        }
        Ok(SOURCES
            .iter()
            .map(|(file_name, _)| SharedString::from(format!("{ICON_DIRECTORY}/{file_name}.svg")))
            .collect())
    }
}

#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Hsla>,
    stroke: Stroke,
    quarter_turns: f32,
}

impl Icon {
    #[must_use]
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: px(16.),
            color: None,
            stroke: Stroke::Regular,
            quarter_turns: 0.,
        }
    }

    #[must_use]
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    #[must_use]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    #[must_use]
    pub fn bold(self, accent: Hsla) -> Self {
        self.color(accent)
    }

    #[must_use]
    pub fn quarter_turns(mut self, turns: f32) -> Self {
        self.quarter_turns = turns;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        let mut glyph = svg()
            .path(icon_path(self.name, self.stroke))
            .size(self.size)
            .flex_none()
            .text_color(color);
        if self.quarter_turns != 0. {
            glyph = glyph.with_transformation(Transformation::rotate(radians(
                self.quarter_turns * std::f32::consts::FRAC_PI_2,
            )));
        }
        glyph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded(path: &str) -> Option<String> {
        IconAssets
            .load(path)
            .unwrap()
            .map(|bytes| String::from_utf8(bytes.into_owned()).unwrap())
    }

    #[test]
    fn a_vendored_icon_loads_with_the_regular_stroke() {
        let svg = loaded("icons/check.svg").unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("stroke-width=\"1.5\""));
        assert!(!svg.contains("stroke-width=\"2\""));
    }

    #[test]
    fn a_stroke_suffix_overrides_the_stroke_width() {
        let svg = loaded("icons/chevron-right@3.svg").unwrap();
        assert!(svg.contains("stroke-width=\"3\""));
        let svg = loaded("icons/chevron-right@2.5.svg").unwrap();
        assert!(svg.contains("stroke-width=\"2.5\""));
    }

    #[test]
    fn an_unknown_icon_is_not_found() {
        assert_eq!(loaded("icons/unicorn.svg"), None);
        assert_eq!(loaded("fonts/Inter-Regular.ttf"), None);
        assert_eq!(loaded("icons/check@fat.svg"), None);
    }

    #[test]
    fn listing_icons_returns_every_vendored_icon_path() {
        let listed = IconAssets.list("icons").unwrap();
        assert!(listed.iter().any(|path| path == "icons/file-text.svg"));
        assert_eq!(listed.len(), IconName::ALL.len());
        assert_eq!(listed.len(), 32);
    }

    #[test]
    fn heavy_and_medium_paths_carry_their_stroke_suffix() {
        assert_eq!(
            icon_path(IconName::ChevronRight, Stroke::Heavy),
            "icons/chevron-right@3.svg"
        );
        assert_eq!(
            icon_path(IconName::ChevronUp, Stroke::Medium),
            "icons/chevron-up@2.5.svg"
        );
        assert_eq!(icon_path(IconName::X, Stroke::Regular), "icons/x.svg");
    }

    #[test]
    fn every_icon_name_resolves_to_a_loadable_asset() {
        for name in IconName::ALL {
            for stroke in [Stroke::Regular, Stroke::Medium, Stroke::Heavy] {
                assert!(loaded(&icon_path(*name, stroke)).is_some(), "{name:?}");
            }
        }
    }

    #[test]
    fn nonsense_stroke_values_are_refused() {
        assert_eq!(loaded("icons/check@NaN.svg"), None);
        assert_eq!(loaded("icons/check@0.svg"), None);
        assert_eq!(loaded("icons/check@9.svg"), None);
        assert_eq!(loaded("icons/check@-1.svg"), None);
    }

    #[test]
    fn listing_anything_but_the_icon_directory_is_empty() {
        assert_eq!(
            IconAssets.list("fonts").unwrap(),
            Vec::<SharedString>::new()
        );
    }
}
