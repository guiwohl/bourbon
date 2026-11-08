const MASK: char = '\u{2022}';

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Mask {
    char_starts: Vec<usize>,
}

impl Mask {
    pub(crate) fn of(text: &str) -> Self {
        Self {
            char_starts: text
                .char_indices()
                .map(|(start, _)| start)
                .chain([text.len()])
                .collect(),
        }
    }

    pub(crate) fn shown(&self) -> String {
        MASK.to_string().repeat(self.char_count())
    }

    pub(crate) fn shown_offset(&self, offset: usize) -> usize {
        self.char_starts.partition_point(|&start| start < offset) * MASK.len_utf8()
    }

    pub(crate) fn text_offset(&self, shown: usize) -> usize {
        let index = (shown / MASK.len_utf8()).min(self.char_count());
        self.char_starts[index]
    }

    fn char_count(&self) -> usize {
        self.char_starts.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_character_shows_as_one_bullet() {
        assert_eq!(Mask::of("aé€").shown(), "\u{2022}\u{2022}\u{2022}");
        assert_eq!(Mask::of("").shown(), "");
    }

    #[test]
    fn offsets_map_through_the_bullets_and_back() {
        let mask = Mask::of("aé€");
        let offsets = [0, 1, 3, 6];
        let shown: Vec<usize> = offsets
            .iter()
            .map(|&offset| mask.shown_offset(offset))
            .collect();
        assert_eq!(shown, [0, 3, 6, 9]);
        let back: Vec<usize> = shown
            .iter()
            .map(|&offset| mask.text_offset(offset))
            .collect();
        assert_eq!(back, offsets);
    }

    #[test]
    fn a_shown_offset_inside_or_past_a_bullet_snaps_to_a_character_start() {
        let mask = Mask::of("ab");
        assert_eq!(mask.text_offset(4), 1);
        assert_eq!(mask.text_offset(99), 2);
    }
}
