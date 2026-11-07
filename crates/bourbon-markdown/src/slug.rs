use std::collections::HashSet;

#[derive(Default)]
pub struct Slugger {
    used: HashSet<String>,
}

impl Slugger {
    pub fn slug(&mut self, text: &str) -> String {
        let base: String = text
            .chars()
            .filter_map(|character| match character {
                ' ' => Some('-'),
                '-' | '_' => Some(character),
                other if other.is_alphanumeric() => Some(other),
                _ => None,
            })
            .flat_map(char::to_lowercase)
            .collect();
        let mut unique = base.clone();
        let mut suffix = 0;
        while self.used.contains(&unique) {
            suffix += 1;
            unique = format!("{base}-{suffix}");
        }
        self.used.insert(unique.clone());
        unique
    }
}

#[cfg(test)]
mod tests {
    use super::Slugger;

    #[test]
    fn lowercases_and_hyphenates_spaces() {
        assert_eq!(
            Slugger::default().slug("What the profiler says"),
            "what-the-profiler-says"
        );
    }

    #[test]
    fn drops_punctuation_but_keeps_hyphen_and_underscore() {
        assert_eq!(
            Slugger::default().slug("App hangs: 5,000-row_table!"),
            "app-hangs-5000-row_table"
        );
    }

    #[test]
    fn keeps_unicode_letters() {
        assert_eq!(Slugger::default().slug("Árvore Ünica"), "árvore-ünica");
    }

    #[test]
    fn duplicates_get_numeric_suffixes() {
        let mut slugger = Slugger::default();
        assert_eq!(slugger.slug("Repro"), "repro");
        assert_eq!(slugger.slug("Repro"), "repro-1");
        assert_eq!(slugger.slug("Repro"), "repro-2");
    }

    #[test]
    fn suffix_never_collides_with_an_existing_slug() {
        let mut slugger = Slugger::default();
        assert_eq!(slugger.slug("a-1"), "a-1");
        assert_eq!(slugger.slug("a"), "a");
        assert_eq!(slugger.slug("a"), "a-2");
    }
}
