use std::time::{Duration, Instant};

const RESET_AFTER: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Editor,
    Preview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncState {
    None,
    Editor,
    Preview,
    Disabled,
}

pub struct SyncGuard {
    state: SyncState,
    last_event: Option<Instant>,
}

impl Default for SyncGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncGuard {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SyncState::None,
            last_event: None,
        }
    }

    #[must_use]
    pub fn state(&self, now: Instant) -> SyncState {
        match (self.state, self.last_event) {
            (SyncState::Editor | SyncState::Preview, Some(last))
                if now.duration_since(last) >= RESET_AFTER =>
            {
                SyncState::None
            }
            (state, _) => state,
        }
    }

    pub fn accept(&mut self, pane: Pane, now: Instant) -> bool {
        let driver = match (self.state(now), pane) {
            (SyncState::None | SyncState::Editor, Pane::Editor) => SyncState::Editor,
            (SyncState::None | SyncState::Preview, Pane::Preview) => SyncState::Preview,
            _ => return false,
        };
        self.state = driver;
        self.last_event = Some(now);
        true
    }

    pub fn disable(&mut self) {
        self.state = SyncState::Disabled;
    }

    pub fn enable(&mut self) {
        if self.state == SyncState::Disabled {
            self.state = SyncState::None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Pane, SyncGuard, SyncState};
    use std::time::{Duration, Instant};

    fn after(start: Instant, milliseconds: u64) -> Instant {
        start + Duration::from_millis(milliseconds)
    }

    #[test]
    fn starts_idle_and_first_pane_wins() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        assert_eq!(guard.state(start), SyncState::None);
        assert!(guard.accept(Pane::Editor, start));
        assert_eq!(guard.state(start), SyncState::Editor);
    }

    #[test]
    fn echo_from_the_driven_pane_is_ignored() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        assert!(guard.accept(Pane::Editor, start));
        assert!(!guard.accept(Pane::Preview, after(start, 10)));
        assert_eq!(guard.state(after(start, 10)), SyncState::Editor);
    }

    #[test]
    fn same_pane_keeps_driving_and_refreshes_the_timer() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        assert!(guard.accept(Pane::Preview, start));
        assert!(guard.accept(Pane::Preview, after(start, 80)));
        assert_eq!(guard.state(after(start, 170)), SyncState::Preview);
        assert!(!guard.accept(Pane::Editor, after(start, 170)));
    }

    #[test]
    fn resets_after_one_hundred_milliseconds() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        assert!(guard.accept(Pane::Editor, start));
        assert_eq!(guard.state(after(start, 99)), SyncState::Editor);
        assert_eq!(guard.state(after(start, 100)), SyncState::None);
        assert!(guard.accept(Pane::Preview, after(start, 100)));
        assert_eq!(guard.state(after(start, 100)), SyncState::Preview);
    }

    #[test]
    fn disabled_rejects_everything_and_never_expires() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        guard.disable();
        assert!(!guard.accept(Pane::Editor, start));
        assert!(!guard.accept(Pane::Preview, after(start, 5000)));
        assert_eq!(guard.state(after(start, 5000)), SyncState::Disabled);
    }

    #[test]
    fn enable_returns_to_idle() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        guard.disable();
        guard.enable();
        assert_eq!(guard.state(start), SyncState::None);
        assert!(guard.accept(Pane::Editor, start));
    }

    #[test]
    fn enable_without_disable_keeps_the_active_state() {
        let start = Instant::now();
        let mut guard = SyncGuard::new();
        assert!(guard.accept(Pane::Editor, start));
        guard.enable();
        assert_eq!(guard.state(start), SyncState::Editor);
    }
}
