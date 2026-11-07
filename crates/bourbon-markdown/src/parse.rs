use pulldown_cmark::{Event, Options, Parser};
use std::ops::Range;

pub fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_MATH
        | Options::ENABLE_GFM
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
}

pub fn events(source: &str) -> Vec<(Event<'_>, Range<usize>)> {
    Parser::new_ext(source, options())
        .into_offset_iter()
        .collect()
}
