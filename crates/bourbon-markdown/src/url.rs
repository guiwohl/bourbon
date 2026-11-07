const NOTE_PREFIX: &str = "bourbon://note/";
const ALLOWED_SCHEMES: [&str; 4] = ["http", "https", "mailto", "bourbon"];

pub fn is_safe(href: &str) -> bool {
    let compact: String = href
        .chars()
        .filter(|character| !character.is_whitespace() && !character.is_control())
        .collect();
    match compact.split_once(':') {
        Some((scheme, _)) if is_scheme(scheme) => {
            ALLOWED_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str())
        }
        _ => true,
    }
}

fn is_scheme(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters
            .all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character))
}

pub fn note_id(href: &str) -> Option<&str> {
    href.strip_prefix(NOTE_PREFIX).filter(|id| {
        !id.is_empty()
            && id
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::{is_safe, note_id};

    #[test]
    fn allows_listed_schemes() {
        for href in [
            "http://a.b",
            "https://a.b/c",
            "mailto:x@y.z",
            "bourbon://note/abc123",
            "HTTPS://A.B",
        ] {
            assert!(is_safe(href), "{href}");
        }
    }

    #[test]
    fn allows_relative_and_fragment_urls() {
        for href in ["", "#section", "/path", "a/b.png", "./x", "?q=1", "a/b:c"] {
            assert!(is_safe(href), "{href}");
        }
    }

    #[test]
    fn rejects_other_schemes_even_when_obfuscated() {
        for href in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            " javascript:x",
            "java\nscript:x",
            "data:text/html,x",
            "file:/etc/passwd",
            "vbscript:x",
        ] {
            assert!(!is_safe(href), "{href:?}");
        }
    }

    #[test]
    fn scheme_must_start_with_a_letter() {
        assert!(is_safe("1a:b"));
        assert!(!is_safe("a+b-c.d:x"));
    }

    #[test]
    fn extracts_note_id() {
        assert_eq!(note_id("bourbon://note/abcdefghij12"), Some("abcdefghij12"));
    }

    #[test]
    fn rejects_malformed_note_links() {
        assert_eq!(note_id("bourbon://note/"), None);
        assert_eq!(note_id("bourbon://note/a/b"), None);
        assert_eq!(note_id("bourbon://note/a b"), None);
        assert_eq!(note_id("https://note/abc"), None);
    }
}
