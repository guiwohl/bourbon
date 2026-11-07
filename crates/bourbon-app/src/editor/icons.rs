use gpui_kit::AssetSource as _;
use gpui_kit::assets::{IconName, icon_assets};
use gpui_kit::component::Icon;

icon_assets!(
    EditorIcons,
    [
        Pencil,
        Columns2,
        Briefcase,
        Expand,
        Circle,
        CirclePlay,
        CirclePause
    ]
);

pub(super) fn editor_icon(name: IconName) -> Icon {
    let svg = EditorIcons.load(&name.path()).ok().flatten();
    svg.map_or_else(Icon::empty, |bytes| Icon::empty().data(&bytes))
}
