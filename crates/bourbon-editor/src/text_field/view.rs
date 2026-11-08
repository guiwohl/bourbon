use bourbon_ui::{Colors, Icon, Scale, ThemeExt as _};
use gpui::{
    Context, CursorStyle, Div, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Pixels, Styled as _, Window, div, px, transparent_black,
};

use super::element::TextFieldElement;
use super::keys::TEXT_FIELD_CONTEXT;
use super::{TextField, TextFieldEvent};
use crate::actions;
use crate::motion::Motion;

const FIELD_HEIGHT: f32 = 30.;
const HORIZONTAL_PADDING: f32 = 8.;
const ICON_SIZE: f32 = 16.;
const ICON_OPACITY: f32 = 0.3;

pub(super) fn render(
    field: &TextField,
    window: &mut Window,
    cx: &mut Context<TextField>,
) -> impl IntoElement {
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    let focused = field.focus_handle.is_focused(window);
    let ring_inset = ring_inset(&tokens.scale);
    let root = div()
        .key_context(TEXT_FIELD_CONTEXT)
        .track_focus(&field.focus_handle)
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HORIZONTAL_PADDING))
        .w_full()
        .min_w_0()
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(MouseButton::Left, cx.listener(TextField::on_mouse_down))
        .on_mouse_move(cx.listener(TextField::on_mouse_move))
        .on_mouse_up(MouseButton::Left, cx.listener(TextField::on_mouse_up))
        .on_mouse_up_out(MouseButton::Left, cx.listener(TextField::on_mouse_up));
    let framed = !field.look.borderless;
    let root = if framed {
        root.h(px(FIELD_HEIGHT))
            .px(px(HORIZONTAL_PADDING))
            .rounded(tokens.scale.radius_default)
            .border_1()
            .border_color(border_color(focused, &colors))
    } else {
        root
    };
    with_actions(root, cx)
        .children(field.leading_icon.map(|icon| {
            div()
                .flex_none()
                .opacity(ICON_OPACITY)
                .child(Icon::new(icon).size(px(ICON_SIZE)).color(colors.text))
        }))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(TextFieldElement::new(cx.entity())),
        )
        .children((framed && focused).then(|| {
            div()
                .absolute()
                .top(ring_inset)
                .bottom(ring_inset)
                .left(ring_inset)
                .right(ring_inset)
                .rounded(ring_radius(&tokens.scale))
                .border(tokens.scale.focus_ring_width)
                .border_color(colors.focus_outline)
        }))
}

fn border_color(focused: bool, colors: &Colors) -> Hsla {
    if focused {
        transparent_black()
    } else {
        colors.internal_border
    }
}

fn ring_inset(scale: &Scale) -> Pixels {
    -(scale.border + scale.focus_ring_width)
}

fn ring_radius(scale: &Scale) -> Pixels {
    scale.radius_default - ring_inset(scale)
}

fn with_actions(root: Div, cx: &mut Context<TextField>) -> Div {
    root.on_action(cx.listener(|f, _: &actions::Left, _, cx| f.move_caret(Motion::Left, false, cx)))
        .on_action(
            cx.listener(|f, _: &actions::Right, _, cx| f.move_caret(Motion::Right, false, cx)),
        )
        .on_action(
            cx.listener(|f, _: &actions::SelectLeft, _, cx| f.move_caret(Motion::Left, true, cx)),
        )
        .on_action(
            cx.listener(|f, _: &actions::SelectRight, _, cx| f.move_caret(Motion::Right, true, cx)),
        )
        .on_action(cx.listener(|f, _: &actions::WordLeft, _, cx| {
            f.move_caret(Motion::WordLeft, false, cx);
        }))
        .on_action(cx.listener(|f, _: &actions::WordRight, _, cx| {
            f.move_caret(Motion::WordRight, false, cx);
        }))
        .on_action(cx.listener(|f, _: &actions::SelectWordLeft, _, cx| {
            f.move_caret(Motion::WordLeft, true, cx);
        }))
        .on_action(cx.listener(|f, _: &actions::SelectWordRight, _, cx| {
            f.move_caret(Motion::WordRight, true, cx);
        }))
        .on_action(
            cx.listener(|f, _: &actions::Home, _, cx| f.move_caret(Motion::LineStart, false, cx)),
        )
        .on_action(
            cx.listener(|f, _: &actions::End, _, cx| f.move_caret(Motion::LineEnd, false, cx)),
        )
        .on_action(cx.listener(|f, _: &actions::SelectHome, _, cx| {
            f.move_caret(Motion::LineStart, true, cx);
        }))
        .on_action(
            cx.listener(|f, _: &actions::SelectEnd, _, cx| f.move_caret(Motion::LineEnd, true, cx)),
        )
        .on_action(cx.listener(|f, _: &actions::Backspace, _, cx| f.backspace(cx)))
        .on_action(cx.listener(|f, _: &actions::Delete, _, cx| f.delete(cx)))
        .on_action(
            cx.listener(|f, _: &actions::DeleteWordBackward, _, cx| f.delete_word_backward(cx)),
        )
        .on_action(
            cx.listener(|f, _: &actions::DeleteWordForward, _, cx| f.delete_word_forward(cx)),
        )
        .on_action(cx.listener(|_f, _: &actions::Enter, _, cx| cx.emit(TextFieldEvent::Submit)))
        .on_action(cx.listener(|_f, _: &actions::Cancel, _, cx| cx.emit(TextFieldEvent::Cancel)))
        .on_action(cx.listener(|f, _: &actions::Undo, _, cx| f.undo(cx)))
        .on_action(cx.listener(|f, _: &actions::Redo, _, cx| f.redo(cx)))
        .on_action(cx.listener(|f, _: &actions::SelectAll, _, cx| f.select_all(cx)))
        .on_action(cx.listener(|f, _: &actions::Copy, _, cx| {
            f.copy(cx);
        }))
        .on_action(cx.listener(|f, _: &actions::Cut, _, cx| f.cut(cx)))
        .on_action(cx.listener(|f, _: &actions::Paste, _, cx| f.paste(cx)))
}

#[cfg(test)]
mod tests {
    use bourbon_ui::Tokens;

    use super::*;

    #[test]
    fn the_resting_border_gives_way_to_the_focus_ring() {
        let colors = Tokens::light().colors;
        assert_eq!(border_color(false, &colors), colors.internal_border);
        assert_eq!(border_color(true, &colors), transparent_black());
    }

    #[test]
    fn the_focus_ring_sits_just_outside_the_border() {
        assert_eq!(ring_inset(&Tokens::light().scale), px(-3.0));
        assert_eq!(ring_radius(&Tokens::light().scale), px(10.0));
    }
}
