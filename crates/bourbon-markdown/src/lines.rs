pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let line_breaks = source.match_indices('\n').map(|(index, _)| index + 1);
        Self {
            starts: std::iter::once(0).chain(line_breaks).collect(),
        }
    }

    pub fn line_of(&self, offset: usize) -> usize {
        self.starts.partition_point(|start| *start <= offset) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    #[test]
    fn maps_offsets_to_zero_based_lines() {
        let index = LineIndex::new("ab\ncd\n\nef");
        assert_eq!(index.line_of(0), 0);
        assert_eq!(index.line_of(2), 0);
        assert_eq!(index.line_of(3), 1);
        assert_eq!(index.line_of(5), 1);
        assert_eq!(index.line_of(6), 2);
        assert_eq!(index.line_of(7), 3);
        assert_eq!(index.line_of(9), 3);
    }

    #[test]
    fn offset_past_the_end_maps_to_last_line() {
        let index = LineIndex::new("a\nb");
        assert_eq!(index.line_of(100), 1);
    }

    #[test]
    fn empty_source_has_one_line() {
        assert_eq!(LineIndex::new("").line_of(0), 0);
    }
}
