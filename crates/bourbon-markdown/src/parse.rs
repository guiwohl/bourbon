use pulldown_cmark::{Event, Options, Parser};
use std::ops::Range;

pub fn options() -> Options {
    Options::ENABLE_TABLES
        .union(Options::ENABLE_TASKLISTS)
        .union(Options::ENABLE_STRIKETHROUGH)
        .union(Options::ENABLE_FOOTNOTES)
        .union(Options::ENABLE_MATH)
        .union(Options::ENABLE_GFM)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
}

pub fn events(source: &str) -> Vec<(Event<'_>, Range<usize>)> {
    Parser::new_ext(source, options())
        .into_offset_iter()
        .collect()
}
