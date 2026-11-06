pub(crate) fn next(current: usize, length: usize) -> usize {
    (current + 1).min(length.saturating_sub(1))
}

pub(crate) fn previous(current: usize) -> usize {
    current.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_moves_down_one_row() {
        assert_eq!(next(0, 3), 1);
        assert_eq!(next(1, 3), 2);
    }

    #[test]
    fn next_stops_at_the_last_row() {
        assert_eq!(next(2, 3), 2);
    }

    #[test]
    fn next_with_no_rows_stays_at_zero() {
        assert_eq!(next(0, 0), 0);
    }

    #[test]
    fn next_with_one_row_stays_on_it() {
        assert_eq!(next(0, 1), 0);
    }

    #[test]
    fn previous_moves_up_one_row() {
        assert_eq!(previous(2), 1);
    }

    #[test]
    fn previous_stops_at_the_first_row() {
        assert_eq!(previous(0), 0);
    }
}
