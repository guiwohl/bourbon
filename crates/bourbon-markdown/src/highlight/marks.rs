use std::ops::Range;

pub struct CodeFence {
    pub open: Range<usize>,
    pub info: Range<usize>,
    pub close: Option<Range<usize>>,
}

pub fn trimmed(source: &str, range: &Range<usize>) -> Range<usize> {
    let end = range.end.min(source.len());
    let end = source[..end]
        .trim_end_matches(['\n', '\r'])
        .len()
        .max(range.start);
    range.start..end
}

fn skip_blanks(source: &str, position: usize) -> usize {
    let blanks = source.as_bytes()[position..]
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    position + blanks
}

fn run_length(source: &str, position: usize, byte: u8) -> usize {
    source.as_bytes()[position..]
        .iter()
        .take_while(|candidate| **candidate == byte)
        .count()
}

pub fn heading_mark(source: &str, range: &Range<usize>) -> Range<usize> {
    let start = skip_blanks(source, range.start);
    start..start + run_length(source, start, b'#')
}

pub fn list_marker(source: &str, range: &Range<usize>) -> Range<usize> {
    let start = skip_blanks(source, range.start);
    let digits = source.as_bytes()[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    start..start + digits + 1
}

pub fn math_marks(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let delimiter = run_length(source, range.start, b'$');
    let inner = range.start + delimiter..range.end - delimiter;
    let mut marks = Vec::new();
    marks.push(range.start..inner.start);
    let mut position = inner.start;
    for _ in 0..inner.len() {
        let Some(offset) = source[position..inner.end].find('\\') else {
            break;
        };
        let start = position + offset;
        position = start + 1 + command_name_len(&source[start + 1..inner.end]);
        marks.push(start..position);
    }
    marks.push(inner.end..range.end);
    marks
}

fn command_name_len(after_backslash: &str) -> usize {
    let letters = after_backslash
        .bytes()
        .take_while(u8::is_ascii_alphabetic)
        .count();
    if letters > 0 {
        return letters;
    }
    after_backslash.chars().next().map_or(0, char::len_utf8)
}

pub fn quote_marks(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let mut marks = Vec::new();
    let mut line_start = range.start;
    for line in source[range.clone()].split_inclusive('\n') {
        let prefix = line
            .bytes()
            .enumerate()
            .take_while(|(_, byte)| matches!(byte, b'>' | b' ' | b'\t'));
        marks.extend(
            prefix
                .filter(|(_, byte)| *byte == b'>')
                .map(|(index, _)| line_start + index..line_start + index + 1),
        );
        line_start += line.len();
    }
    marks
}

pub fn alert_marker(source: &str, range: &Range<usize>) -> Option<Range<usize>> {
    let line_end = source[range.start..range.end]
        .find('\n')
        .map_or(range.end, |newline| range.start + newline);
    let open = range.start + source[range.start..line_end].find("[!")?;
    let close = open + source[open..line_end].find(']')?;
    Some(open..close + 1)
}

pub fn delimiters(range: &Range<usize>, length: usize) -> [Range<usize>; 2] {
    [
        range.start..range.start + length,
        range.end.saturating_sub(length)..range.end,
    ]
}

pub fn strikethrough_delimiters(source: &str, range: &Range<usize>) -> [Range<usize>; 2] {
    delimiters(range, run_length(source, range.start, b'~'))
}

pub fn code_delimiters(source: &str, range: &Range<usize>) -> [Range<usize>; 2] {
    delimiters(range, run_length(source, range.start, b'`'))
}

pub fn code_fence(source: &str, range: &Range<usize>) -> Option<CodeFence> {
    let start = skip_blanks(source, range.start);
    let fence_byte = *source.as_bytes().get(start)?;
    let length = run_length(source, start, fence_byte);
    let open = start..start + length;
    let first_line_end = source[start..range.end]
        .find('\n')
        .map_or(range.end, |newline| start + newline);
    let info_text = &source[open.end..first_line_end];
    let info_start = open.end + (info_text.len() - info_text.trim_start().len());
    let info_end = open.end + info_text.trim_end().len();
    Some(CodeFence {
        info: info_start..info_end,
        close: closing_fence(source, range, first_line_end, fence_byte),
        open,
    })
}

fn closing_fence(
    source: &str,
    range: &Range<usize>,
    first_line_end: usize,
    fence_byte: u8,
) -> Option<Range<usize>> {
    let body = trimmed(source, range);
    let last_line_start = first_line_end + source[first_line_end..body.end].rfind('\n')? + 1;
    let start = skip_blanks(source, last_line_start);
    let length = run_length(source, start, fence_byte);
    let only_fence = source[start + length..body.end].trim().is_empty();
    (length >= 3 && only_fence).then_some(start..start + length)
}

pub struct LinkParts {
    pub marks: Vec<Range<usize>>,
    pub url: Range<usize>,
}

pub fn link_parts(
    source: &str,
    range: &Range<usize>,
    open_length: usize,
    text_end: usize,
) -> LinkParts {
    let mut marks = Vec::with_capacity(4);
    marks.push(range.start..range.start + open_length);
    let mut url = 0..0;
    let tail = source.get(text_end..range.end).unwrap_or_default();
    if let Some(inner) = tail
        .strip_prefix("](")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let url_start = text_end + 2 + (inner.len() - inner.trim_start().len());
        let url_end = text_end + 2 + inner.trim_end().len();
        marks.extend([text_end..text_end + 2, range.end - 1..range.end]);
        url = url_start..url_end;
    } else if tail.starts_with(']') {
        marks.push(text_end..range.end);
    }
    LinkParts { marks, url }
}

pub fn line_after(source: &str, position: usize) -> Option<Range<usize>> {
    let newline = source
        .match_indices('\n')
        .map(|(index, _)| index)
        .find(|index| *index >= position)?;
    let start = newline + 1;
    let end = source[start..]
        .find('\n')
        .map_or(source.len(), |newline| start + newline);
    Some(start..end)
}
