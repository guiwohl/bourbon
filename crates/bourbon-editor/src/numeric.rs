#[allow(clippy::cast_precision_loss)]
pub fn count_f32(count: usize) -> f32 {
    count as f32
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn row_from_ratio(ratio: f32) -> usize {
    ratio.max(0.0).floor() as usize
}

pub fn signed(count: usize) -> i64 {
    i64::try_from(count).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_come_from_the_floor_of_the_ratio() {
        assert_eq!(row_from_ratio(2.9), 2);
        assert_eq!(row_from_ratio(0.0), 0);
    }

    #[test]
    fn negative_ratios_are_row_zero() {
        assert_eq!(row_from_ratio(-3.2), 0);
    }

    #[test]
    fn counts_convert_exactly_for_small_values() {
        assert_eq!(count_f32(12), 12.0);
        assert_eq!(signed(12), 12);
    }
}
