use gpui::{Pixels, Point, ShapedLine, px};

#[derive(Default)]
pub(crate) struct LineGeometry {
    pub(crate) line: ShapedLine,
    pub(crate) shows_placeholder: bool,
    pub(crate) origin: Point<Pixels>,
    pub(crate) line_height: Pixels,
    pub(crate) scroll_x: Pixels,
}

impl LineGeometry {
    pub(crate) fn x_for_offset(&self, offset: usize) -> Pixels {
        let local = if self.shows_placeholder {
            px(0.0)
        } else {
            self.line.x_for_index(offset)
        };
        self.origin.x + local - self.scroll_x
    }

    pub(crate) fn offset_for_x(&self, x: Pixels) -> usize {
        if self.shows_placeholder {
            return 0;
        }
        self.line
            .closest_index_for_x(x - self.origin.x + self.scroll_x)
    }
}

pub(crate) fn revealing_scroll(
    scroll_x: Pixels,
    caret_x: Pixels,
    content_width: Pixels,
    viewport_width: Pixels,
    caret_width: Pixels,
) -> Pixels {
    let visible_width = (viewport_width - caret_width).max(px(0.0));
    let max_scroll = (content_width - visible_width).max(px(0.0));
    let revealed = if caret_x < scroll_x {
        caret_x
    } else if caret_x > scroll_x + visible_width {
        caret_x - visible_width
    } else {
        scroll_x
    };
    revealed.min(max_scroll).max(px(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reveal(scroll: f32, caret: f32, content: f32) -> f32 {
        f32::from(revealing_scroll(
            px(scroll),
            px(caret),
            px(content),
            px(101.0),
            px(1.0),
        ))
    }

    #[test]
    fn a_caret_inside_the_viewport_keeps_the_scroll() {
        assert_eq!(reveal(20.0, 60.0, 300.0), 20.0);
        assert_eq!(reveal(20.0, 20.0, 300.0), 20.0);
        assert_eq!(reveal(20.0, 120.0, 300.0), 20.0);
    }

    #[test]
    fn a_caret_left_of_the_viewport_scrolls_back_to_it() {
        assert_eq!(reveal(50.0, 30.0, 300.0), 30.0);
    }

    #[test]
    fn a_caret_right_of_the_viewport_scrolls_until_it_is_the_last_visible_pixel() {
        assert_eq!(reveal(0.0, 150.0, 300.0), 50.0);
    }

    #[test]
    fn the_scroll_never_passes_the_end_of_the_text() {
        assert_eq!(reveal(250.0, 260.0, 260.0), 160.0);
    }

    #[test]
    fn short_text_never_scrolls() {
        assert_eq!(reveal(40.0, 50.0, 60.0), 0.0);
    }
}
