#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Forward,
    Backward,
}

#[must_use]
pub fn next_enabled(enabled: &[bool], current: Option<usize>, step: Step) -> Option<usize> {
    let count = enabled.len();
    if count == 0 {
        return None;
    }
    let origin = match (current, step) {
        (Some(index), _) => index.min(count - 1),
        (None, Step::Forward) => count - 1,
        (None, Step::Backward) => 0,
    };
    (1..=count)
        .map(|distance| match step {
            Step::Forward => (origin + distance) % count,
            Step::Backward => (origin + count - distance) % count,
        })
        .find(|index| enabled[*index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_from_nothing_picks_the_first_enabled_item() {
        assert_eq!(
            next_enabled(&[false, true, true], None, Step::Forward),
            Some(1)
        );
    }

    #[test]
    fn backward_from_nothing_picks_the_last_enabled_item() {
        assert_eq!(
            next_enabled(&[true, true, false], None, Step::Backward),
            Some(1)
        );
    }

    #[test]
    fn forward_skips_disabled_items() {
        assert_eq!(
            next_enabled(&[true, false, false, true], Some(0), Step::Forward),
            Some(3)
        );
    }

    #[test]
    fn forward_wraps_to_the_start() {
        assert_eq!(
            next_enabled(&[true, true, false], Some(1), Step::Forward),
            Some(0)
        );
    }

    #[test]
    fn backward_wraps_to_the_end() {
        assert_eq!(
            next_enabled(&[true, true, true], Some(0), Step::Backward),
            Some(2)
        );
    }

    #[test]
    fn nothing_is_chosen_when_every_item_is_disabled() {
        assert_eq!(next_enabled(&[false, false], Some(0), Step::Forward), None);
        assert_eq!(next_enabled(&[], None, Step::Backward), None);
    }

    #[test]
    fn a_single_enabled_item_stays_selected() {
        assert_eq!(
            next_enabled(&[false, true], Some(1), Step::Forward),
            Some(1)
        );
    }
}
