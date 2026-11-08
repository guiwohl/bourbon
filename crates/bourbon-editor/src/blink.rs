use std::time::Duration;

use gpui::Context;

pub(crate) const BLINK_INTERVAL: Duration = Duration::from_millis(530);

pub(crate) fn schedule_blink<T: 'static>(
    epoch: u64,
    cx: &mut Context<T>,
    tick: fn(&mut T, u64, &mut Context<T>),
) {
    cx.spawn(async move |owner, cx| {
        cx.background_executor().timer(BLINK_INTERVAL).await;
        owner.update(cx, |owner, cx| tick(owner, epoch, cx)).ok();
    })
    .detach();
}
