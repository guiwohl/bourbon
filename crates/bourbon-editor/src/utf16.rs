use std::ops::Range;

pub fn offset_to_utf16(text: &str, offset: usize) -> usize {
    let end = offset.min(text.len());
    text.char_indices()
        .take_while(|(index, _)| *index < end)
        .map(|(_, character)| character.len_utf16())
        .sum()
}

pub fn offset_from_utf16(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        units += character.len_utf16();
        if units > offset {
            return index;
        }
    }
    text.len()
}

pub fn range_to_utf16(text: &str, range: &Range<usize>) -> Range<usize> {
    offset_to_utf16(text, range.start)..offset_to_utf16(text, range.end)
}

pub fn range_from_utf16(text: &str, range: &Range<usize>) -> Range<usize> {
    offset_from_utf16(text, range.start)..offset_from_utf16(text, range.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn ascii_offsets_are_identical() {
        assert_eq!(offset_to_utf16("hello", 3), 3);
        assert_eq!(offset_from_utf16("hello", 3), 3);
    }

    #[test]
    fn astral_characters_count_as_two_units() {
        let text = "e🙂漢";
        assert_eq!(offset_to_utf16(text, 1), 1);
        assert_eq!(offset_to_utf16(text, 5), 3);
        assert_eq!(offset_to_utf16(text, 8), 4);
        assert_eq!(offset_from_utf16(text, 3), 5);
        assert_eq!(offset_from_utf16(text, 4), 8);
    }

    #[test]
    fn offsets_past_the_end_are_clamped() {
        assert_eq!(offset_to_utf16("ab", 99), 2);
        assert_eq!(offset_from_utf16("e🙂", 99), 5);
    }

    #[test]
    fn middle_of_a_surrogate_pair_rounds_down() {
        assert_eq!(offset_from_utf16("e🙂", 2), 1);
    }

    #[test]
    fn ranges_convert_both_ways() {
        let text = "a🙂b";
        assert_eq!(range_to_utf16(text, &(1..5)), 1..3);
        assert_eq!(range_from_utf16(text, &(1..3)), 1..5);
    }

    proptest! {
        #[test]
        fn round_trips_on_char_boundaries(text in "\\PC{0,40}") {
            for (offset, _) in text.char_indices().chain([(text.len(), ' ')]) {
                prop_assert_eq!(offset_from_utf16(&text, offset_to_utf16(&text, offset)), offset);
            }
        }

        #[test]
        fn conversion_is_monotonic(text in "\\PC{0,40}", a in 0usize..60, b in 0usize..60) {
            let (low, high) = (a.min(b), a.max(b));
            prop_assert!(offset_from_utf16(&text, low) <= offset_from_utf16(&text, high));
        }

        #[test]
        fn from_utf16_always_lands_on_a_char_boundary(text in "\\PC{0,40}", offset in 0usize..100) {
            prop_assert!(text.is_char_boundary(offset_from_utf16(&text, offset)));
        }
    }
}
