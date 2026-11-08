use std::time::Duration;

use gpui::{Context, Task};

pub(crate) const AUTOSAVE_DELAY: Duration = Duration::from_millis(500);

pub(crate) fn after_delay<View: 'static>(
    cx: &mut Context<View>,
    fire: impl FnOnce(&mut View, &mut Context<View>) + 'static,
) -> Task<()> {
    cx.spawn(async move |view, cx| {
        cx.background_executor().timer(AUTOSAVE_DELAY).await;
        view.update(cx, fire).ok();
    })
}
