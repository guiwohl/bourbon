use std::borrow::Cow;

const MAX_SLUG_LENGTH: usize = 100;
const MAX_FOLDER_NAME_BYTES: usize = 200;
const UNTITLED: &str = "untitled";

pub fn slug_of(title: &str) -> String {
    let mut slug = String::new();
    let mut separator_pending = false;
    for character in title.chars().flat_map(char::to_lowercase) {
        if let Some(text) = slug_text_of(character) {
            if separator_pending && !slug.is_empty() {
                slug.push('-');
            }
            separator_pending = false;
            slug.push_str(&text);
        } else if !character.is_alphabetic() {
            separator_pending = true;
        }
    }
    slug.truncate(MAX_SLUG_LENGTH);
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        UNTITLED.to_owned()
    } else {
        slug.to_owned()
    }
}

pub fn folder_name_of(notebook_name: &str) -> String {
    let replaced: String = notebook_name
        .chars()
        .map(|character| {
            if character == '/' || character == '\0' {
                '-'
            } else {
                character
            }
        })
        .collect();
    let trimmed = replaced.trim();
    if matches!(trimmed, "" | "." | "..") {
        return UNTITLED.to_owned();
    }
    let mut cut = String::new();
    for character in trimmed.chars() {
        if cut.len() + character.len_utf8() > MAX_FOLDER_NAME_BYTES {
            break;
        }
        cut.push(character);
    }
    cut.trim_end().to_owned()
}

fn slug_text_of(character: char) -> Option<Cow<'static, str>> {
    if character.is_ascii_alphanumeric() {
        return Some(Cow::Owned(character.to_string()));
    }
    let text = match character {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
        'ç' => "c",
        'è' | 'é' | 'ê' | 'ë' => "e",
        'ì' | 'í' | 'î' | 'ï' => "i",
        'ñ' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
        'ù' | 'ú' | 'û' | 'ü' => "u",
        'ý' | 'ÿ' => "y",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        _ => return None,
    };
    Some(Cow::Borrowed(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_lowercases_and_joins_words_with_dashes() {
        assert_eq!(slug_of("Hello World"), "hello-world");
    }

    #[test]
    fn slug_collapses_runs_of_other_characters() {
        assert_eq!(slug_of("a  --  b!?c"), "a-b-c");
    }

    #[test]
    fn slug_trims_leading_and_trailing_separators() {
        assert_eq!(slug_of("  ...Hi!!  "), "hi");
    }

    #[test]
    fn slug_keeps_digits() {
        assert_eq!(slug_of("Plan 2026 v2"), "plan-2026-v2");
    }

    #[test]
    fn slug_transliterates_common_latin_letters() {
        assert_eq!(
            slug_of("Café Crème Ação Straße Œuvre"),
            "cafe-creme-acao-strasse-oeuvre"
        );
        assert_eq!(slug_of("ÀÉÎÕÜ"), "aeiou");
    }

    #[test]
    fn every_transliterated_letter_maps_to_its_plain_form() {
        let table = [
            ("àáâãäå", "a"),
            ("ç", "c"),
            ("èéêë", "e"),
            ("ìíîï", "i"),
            ("ñ", "n"),
            ("òóôõöø", "o"),
            ("ùúûü", "u"),
            ("ýÿ", "y"),
            ("ß", "ss"),
            ("æ", "ae"),
            ("œ", "oe"),
        ];
        for (letters, plain) in table {
            for letter in letters.chars() {
                assert_eq!(slug_of(&format!("x{letter}x")), format!("x{plain}x"));
            }
        }
    }

    #[test]
    fn slug_drops_letters_it_cannot_transliterate() {
        assert_eq!(slug_of("日本語 note"), "note");
        assert_eq!(slug_of("a日b"), "ab");
    }

    #[test]
    fn slug_of_nothing_usable_is_untitled() {
        assert_eq!(slug_of(""), "untitled");
        assert_eq!(slug_of("  !!! "), "untitled");
        assert_eq!(slug_of("日本語"), "untitled");
    }

    #[test]
    fn slug_is_cut_at_one_hundred_characters_without_a_trailing_dash() {
        let title = format!("{} tail", "a".repeat(99));
        assert_eq!(slug_of(&title), "a".repeat(99));
        assert_eq!(slug_of(&"b".repeat(150)), "b".repeat(100));
    }

    #[test]
    fn folder_name_keeps_the_notebook_name() {
        assert_eq!(folder_name_of("Work Notes"), "Work Notes");
    }

    #[test]
    fn folder_name_replaces_slashes_and_nul() {
        assert_eq!(folder_name_of("a/b\0c"), "a-b-c");
    }

    #[test]
    fn folder_name_is_trimmed() {
        assert_eq!(folder_name_of("  Work \t"), "Work");
    }

    #[test]
    fn folder_name_never_escapes_the_parent() {
        assert_eq!(folder_name_of(".."), "untitled");
        assert_eq!(folder_name_of(" . "), "untitled");
    }

    #[test]
    fn empty_folder_name_is_untitled() {
        assert_eq!(folder_name_of(""), "untitled");
        assert_eq!(folder_name_of(" \t "), "untitled");
    }

    #[test]
    fn long_folder_names_are_cut_on_a_character_boundary() {
        let name = "ç".repeat(150);
        let cut = folder_name_of(&name);
        assert_eq!(cut, "ç".repeat(100));
    }
}
