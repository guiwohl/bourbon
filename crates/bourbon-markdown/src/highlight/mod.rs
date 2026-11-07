mod canvas;
mod marks;
mod walk;

use crate::parse;
use crate::token::HighlightSpan;
use walk::Walk;

#[must_use]
pub fn highlight(source: &str) -> Vec<HighlightSpan> {
    let mut walk = Walk::new(source);
    for (event, range) in parse::events(source) {
        walk.visit(&event, range);
    }
    walk.finish()
}

#[cfg(test)]
mod tests;
