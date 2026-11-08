use std::ops::Range;

use regex::{Regex, RegexBuilder};

use crate::change::{Change, ChangeSet};
use crate::selection::{Selection, SelectionRange};
use crate::state::{EditorState, Transaction, TransactionKind};
use crate::text::Text;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchQuery {
    pub pattern: String,
    pub match_case: bool,
    pub regex: bool,
    pub whole_word: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPattern;

#[derive(Debug, Clone)]
pub struct Search {
    matcher: Option<Regex>,
    whole_word: bool,
    expands_captures: bool,
}

impl Search {
    pub fn new(query: &SearchQuery) -> Result<Self, InvalidPattern> {
        let matcher = if query.pattern.is_empty() {
            None
        } else {
            let source = if query.regex {
                query.pattern.clone()
            } else {
                regex::escape(&query.pattern)
            };
            let compiled = RegexBuilder::new(&source)
                .case_insensitive(!query.match_case)
                .multi_line(true)
                .build()
                .map_err(|_| InvalidPattern)?;
            Some(compiled)
        };
        Ok(Self {
            matcher,
            whole_word: query.whole_word,
            expands_captures: query.regex,
        })
    }

    pub fn find_all(&self, text: &Text) -> Vec<Range<usize>> {
        let Some(pattern) = &self.matcher else {
            return Vec::new();
        };
        let haystack = text.to_string();
        let mut matches = Vec::new();
        let mut position = Some(0);
        while let Some(start) = position {
            let Some(found) = pattern.find_at(&haystack, start) else {
                break;
            };
            if !found.is_empty() && self.accepts(&haystack, found.range()) {
                matches.push(found.range());
                position = Some(found.end());
            } else {
                position = next_char_boundary(&haystack, found.start());
            }
        }
        matches
    }

    pub fn replace_all(&self, state: &EditorState, replacement: &str) -> Option<Transaction> {
        let haystack = state.text.to_string();
        let changes: Vec<Change> = self
            .find_all(&state.text)
            .into_iter()
            .map(|range| {
                let text = self.replacement_for(&haystack, &range, replacement);
                Change::replace(range, text)
            })
            .collect();
        if changes.is_empty() {
            return None;
        }
        let changes = ChangeSet::new(changes);
        let selection = state.selection.map(&changes);
        Some(command(changes, selection))
    }

    pub fn replace_match(
        &self,
        state: &EditorState,
        range: Range<usize>,
        replacement: &str,
    ) -> Option<Transaction> {
        if !self.find_all(&state.text).contains(&range) {
            return None;
        }
        let text = self.replacement_for(&state.text.to_string(), &range, replacement);
        let caret = range.start + text.len();
        let changes = ChangeSet::new(vec![Change::replace(range, text)]);
        Some(command(changes, Selection::cursor(caret)))
    }

    fn accepts(&self, haystack: &str, range: Range<usize>) -> bool {
        if !self.whole_word {
            return true;
        }
        let before = haystack[..range.start].chars().next_back();
        let after = haystack[range.end..].chars().next();
        !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
    }

    fn replacement_for(&self, haystack: &str, range: &Range<usize>, replacement: &str) -> String {
        let captures = self
            .matcher
            .as_ref()
            .filter(|_| self.expands_captures)
            .and_then(|matcher| matcher.captures_at(haystack, range.start));
        match captures {
            Some(captures) => {
                let mut expanded = String::new();
                captures.expand(replacement, &mut expanded);
                expanded
            }
            None => replacement.to_owned(),
        }
    }
}

fn command(changes: ChangeSet, selection: Selection) -> Transaction {
    let mut transaction = Transaction::new(changes, selection, TransactionKind::Command);
    transaction.isolate = true;
    transaction
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn next_char_boundary(haystack: &str, position: usize) -> Option<usize> {
    haystack[position..]
        .chars()
        .next()
        .map(|character| position + character.len_utf8())
}

pub fn next_match(matches: &[Range<usize>], selection: SelectionRange) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }
    let after = matches
        .iter()
        .position(|found| found.start >= selection.to());
    Some(after.unwrap_or(0))
}

pub fn previous_match(matches: &[Range<usize>], selection: SelectionRange) -> Option<usize> {
    let before = matches
        .iter()
        .rposition(|found| found.end <= selection.from());
    before.or_else(|| matches.len().checked_sub(1))
}

pub fn current_match(matches: &[Range<usize>], selection: SelectionRange) -> Option<usize> {
    matches
        .iter()
        .position(|found| found.start == selection.from() && found.end == selection.to())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{render, state};

    fn query(pattern: &str) -> SearchQuery {
        SearchQuery {
            pattern: pattern.to_owned(),
            ..SearchQuery::default()
        }
    }

    fn find(query: &SearchQuery, text: &str) -> Vec<Range<usize>> {
        Search::new(query).unwrap().find_all(&Text::new(text))
    }

    #[test]
    fn an_empty_pattern_finds_nothing() {
        assert_eq!(find(&query(""), "abc"), Vec::<Range<usize>>::new());
    }

    #[test]
    fn plain_search_ignores_case_by_default() {
        assert_eq!(find(&query("ab"), "ab AB aB"), vec![0..2, 3..5, 6..8]);
    }

    #[test]
    fn match_case_keeps_only_the_exact_spelling() {
        let exact = SearchQuery {
            match_case: true,
            ..query("ab")
        };
        assert_eq!(find(&exact, "ab AB aB"), vec![0..2]);
    }

    #[test]
    fn plain_search_treats_regex_characters_literally() {
        assert_eq!(find(&query("a.c"), "abc a.c"), vec![4..7]);
    }

    #[test]
    fn regex_search_uses_the_pattern() {
        let pattern = SearchQuery {
            regex: true,
            ..query("a.c")
        };
        assert_eq!(find(&pattern, "abc a.c"), vec![0..3, 4..7]);
    }

    #[test]
    fn regex_anchors_work_per_line() {
        let pattern = SearchQuery {
            regex: true,
            ..query("^x")
        };
        assert_eq!(find(&pattern, "x\nx\nax"), vec![0..1, 2..3]);
    }

    #[test]
    fn an_invalid_regex_is_reported() {
        let pattern = SearchQuery {
            regex: true,
            ..query("(")
        };
        assert_eq!(Search::new(&pattern).unwrap_err(), InvalidPattern);
    }

    #[test]
    fn empty_regex_matches_are_skipped_and_the_search_ends() {
        let pattern = SearchQuery {
            regex: true,
            ..query("x*")
        };
        assert_eq!(find(&pattern, "ab xx é"), vec![3..5]);
        let empty_only = SearchQuery {
            regex: true,
            ..query("^")
        };
        assert_eq!(find(&empty_only, "é\n\nb"), Vec::<Range<usize>>::new());
    }

    #[test]
    fn whole_word_rejects_matches_inside_words() {
        let words = SearchQuery {
            whole_word: true,
            ..query("cat")
        };
        assert_eq!(
            find(&words, "cat concat cats _cat (cat) cat"),
            vec![0..3, 22..25, 27..30]
        );
    }

    #[test]
    fn whole_word_retries_right_after_a_rejected_start() {
        let words = SearchQuery {
            whole_word: true,
            regex: true,
            ..query("a+b?")
        };
        assert_eq!(find(&words, "aab ab"), vec![0..3, 4..6]);
        let overlapping = SearchQuery {
            whole_word: true,
            regex: true,
            ..query("[a-z]a")
        };
        assert_eq!(find(&overlapping, "xba"), Vec::<Range<usize>>::new());
        assert_eq!(find(&overlapping, "ba"), vec![0..2]);
    }

    #[test]
    fn matches_are_byte_ranges_around_multibyte_text() {
        assert_eq!(find(&query("ção"), "ação ÇÃO"), vec![1..6, 7..12]);
    }

    #[test]
    fn replace_all_is_one_command_transaction_mapping_the_selection() {
        let before = state("one two| one");
        let search = Search::new(&query("one")).unwrap();
        let transaction = search.replace_all(&before, "1").unwrap();
        assert_eq!(transaction.kind, TransactionKind::Command);
        assert!(transaction.isolate);
        assert_eq!(render(&before.apply(&transaction)), "1 two| 1");
    }

    #[test]
    fn replace_all_without_matches_does_nothing() {
        let search = Search::new(&query("zzz")).unwrap();
        assert!(search.replace_all(&state("abc|"), "x").is_none());
    }

    #[test]
    fn regex_replacements_expand_capture_groups() {
        let pattern = SearchQuery {
            regex: true,
            ..query(r"(\w+)@(\w+)")
        };
        let search = Search::new(&pattern).unwrap();
        let before = state("|a@b c@d");
        let after = before.apply(&search.replace_all(&before, "$2-$1").unwrap());
        assert_eq!(after.text.to_string(), "b-a d-c");
    }

    #[test]
    fn plain_replacements_keep_dollar_signs() {
        let search = Search::new(&query("a")).unwrap();
        let before = state("|a");
        let after = before.apply(&search.replace_all(&before, "$1").unwrap());
        assert_eq!(after.text.to_string(), "$1");
    }

    #[test]
    fn replacing_one_match_puts_the_caret_after_the_replacement() {
        let before = state("«one» two one");
        let search = Search::new(&query("one")).unwrap();
        let transaction = search.replace_match(&before, 0..3, "three").unwrap();
        assert_eq!(transaction.kind, TransactionKind::Command);
        assert!(transaction.isolate);
        assert_eq!(render(&before.apply(&transaction)), "three| two one");
    }

    #[test]
    fn replacing_a_range_that_is_not_a_match_does_nothing() {
        let search = Search::new(&query("one")).unwrap();
        assert!(
            search
                .replace_match(&state("one two|"), 1..4, "x")
                .is_none()
        );
    }

    #[test]
    fn regex_replace_match_expands_the_groups_of_that_match() {
        let pattern = SearchQuery {
            regex: true,
            ..query(r"(\d)")
        };
        let search = Search::new(&pattern).unwrap();
        let before = state("|1 2");
        let after = before.apply(&search.replace_match(&before, 2..3, "<$1>").unwrap());
        assert_eq!(render(&after), "1 <2>|");
    }

    #[test]
    fn next_match_starts_at_the_selection_end_and_wraps() {
        let matches = [0..2, 4..6, 8..10];
        assert_eq!(next_match(&matches, SelectionRange::cursor(4)), Some(1));
        assert_eq!(next_match(&matches, SelectionRange::new(4, 6)), Some(2));
        assert_eq!(next_match(&matches, SelectionRange::new(8, 10)), Some(0));
        assert_eq!(next_match(&[], SelectionRange::cursor(0)), None);
    }

    #[test]
    fn previous_match_ends_before_the_selection_start_and_wraps() {
        let matches = [0..2, 4..6, 8..10];
        assert_eq!(previous_match(&matches, SelectionRange::new(4, 6)), Some(0));
        assert_eq!(previous_match(&matches, SelectionRange::cursor(6)), Some(1));
        assert_eq!(previous_match(&matches, SelectionRange::new(0, 2)), Some(2));
        assert_eq!(previous_match(&[], SelectionRange::cursor(0)), None);
    }

    #[test]
    fn the_current_match_is_the_one_equal_to_the_selection() {
        let matches = [0..2, 4..6];
        assert_eq!(current_match(&matches, SelectionRange::new(6, 4)), Some(1));
        assert_eq!(current_match(&matches, SelectionRange::new(4, 5)), None);
        assert_eq!(current_match(&matches, SelectionRange::cursor(4)), None);
    }
}
