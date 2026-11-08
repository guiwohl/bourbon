use std::time::Instant;

use bourbon_markdown::{Anchor, Pane, SyncGuard, line_for_preview_offset, preview_offset_for_line};

#[derive(Default)]
pub(crate) struct ScrollSync {
    guard: SyncGuard,
}

impl ScrollSync {
    pub(crate) fn preview_offset_for(
        &mut self,
        editor_line: f32,
        anchors: &[Anchor],
        now: Instant,
    ) -> Option<f32> {
        self.guard
            .accept(Pane::Editor, now)
            .then(|| preview_offset_for_line(anchors, editor_line))
    }

    pub(crate) fn editor_line_for(
        &mut self,
        preview_offset: f32,
        anchors: &[Anchor],
        now: Instant,
    ) -> Option<f32> {
        self.guard
            .accept(Pane::Preview, now)
            .then(|| line_for_preview_offset(anchors, preview_offset))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn anchors() -> Vec<Anchor> {
        vec![
            Anchor {
                line_start: 0,
                line_end: 0,
                top: 10.0,
                height: 20.0,
            },
            Anchor {
                line_start: 4,
                line_end: 4,
                top: 110.0,
                height: 20.0,
            },
        ]
    }

    #[test]
    fn the_editor_drives_the_preview_through_the_line_mapping() {
        let mut sync = ScrollSync::default();
        let offset = sync.preview_offset_for(4.0, &anchors(), Instant::now());
        assert_eq!(offset, Some(100.0));
    }

    #[test]
    fn the_preview_drives_the_editor_through_the_offset_mapping() {
        let mut sync = ScrollSync::default();
        let line = sync.editor_line_for(110.0, &anchors(), Instant::now());
        assert_eq!(line, Some(4.0));
    }

    #[test]
    fn the_echo_of_the_driven_pane_is_ignored() {
        let mut sync = ScrollSync::default();
        let now = Instant::now();
        assert!(sync.preview_offset_for(1.0, &anchors(), now).is_some());
        assert_eq!(sync.editor_line_for(50.0, &anchors(), now), None);
    }

    #[test]
    fn the_other_pane_drives_again_after_the_reset_delay() {
        let mut sync = ScrollSync::default();
        let now = Instant::now();
        sync.preview_offset_for(1.0, &anchors(), now);
        let later = now + Duration::from_millis(150);
        assert!(sync.editor_line_for(50.0, &anchors(), later).is_some());
    }
}
