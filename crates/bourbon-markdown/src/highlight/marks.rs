use std::ops::Range;

pub struct CodeFence {
    pub open: Range<usize>,
    pub info: Option<Range<usize>>,
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

fn skip_blanks(source: &str, mut position: usize) -> usize {
    while matches!(source.as_bytes().get(position), Some(b' ' | b'\t')) {
        position += 1;
    }
    position
}

fn run_length(source: &str, position: usize, byte: u8) -> usize {
    source.as_bytes()[position..]
        .iter()
        .take_while(|candidate| **candidate == byte)
        .count()
}

pub fn heading_mark(source: &str, range: &Range<usize>) -> Option<Range<usize>> {
    let start = skip_blanks(source, range.start);
    let length = run_length(source, start, b'#');
    (1..=6).contains(&length).then_some(start..start + length)
}

pub fn list_marker(source: &str, range: &Range<usize>) -> Option<Range<usize>> {
    let start = skip_blanks(source, range.start);
    let bytes = source.as_bytes();
    match bytes.get(start)? {
        b'-' | b'*' | b'+' => Some(start..start + 1),
        b'0'..=b'9' => {
            let digits = bytes[start..]
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count();
            matches!(bytes.get(start + digits), Some(b'.' | b')'))
                .then_some(start..start + digits + 1)
        }
        _ => None,
    }
}

pub fn quote_marks(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let mut marks = Vec::new();
    let mut line_start = range.start;
    while line_start < range.end {
        let mut position = skip_blanks(source, line_start);
        while source.as_bytes().get(position) == Some(&b'>') {
            marks.push(position..position + 1);
            position = skip_blanks(source, position + 1);
        }
        match source[line_start..range.end].find('\n') {
            Some(newline) => line_start += newline + 1,
            None => break,
        }
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

pub fn delimiters(source: &str, range: &Range<usize>, length: usize) -> Vec<Range<usize>> {
    let length = length.min(range.len() / 2);
    let bounded = range.start + length <= range.end && length > 0;
    if bounded
        && source.is_char_boundary(range.start + length)
        && source.is_char_boundary(range.end - length)
    {
        vec![
            range.start..range.start + length,
            range.end - length..range.end,
        ]
    } else {
        Vec::new()
    }
}

pub fn strikethrough_delimiters(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let length = run_length(source, range.start, b'~');
    delimiters(source, range, length)
}

pub fn code_delimiters(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let length = run_length(source, range.start, b'`');
    delimiters(source, range, length)
}

pub fn code_fence(source: &str, range: &Range<usize>) -> Option<CodeFence> {
    let start = skip_blanks(source, range.start);
    let fence_byte = *source.as_bytes().get(start)?;
    let length = run_length(source, start, fence_byte);
    if !matches!(fence_byte, b'`' | b'~') || length < 3 {
        return None;
    }
    let open = start..start + length;
    let first_line_end = source[start..range.end]
        .find('\n')
        .map_or(range.end, |newline| start + newline);
    let info_text = &source[open.end..first_line_end];
    let info_start = open.end + (info_text.len() - info_text.trim_start().len());
    let info_end = open.end + info_text.trim_end().len();
    Some(CodeFence {
        info: (info_start < info_end).then_some(info_start..info_end),
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
    let last_line_start = source[..body.end].rfind('\n').map(|newline| newline + 1)?;
    if last_line_start <= first_line_end {
        return None;
    }
    let start = skip_blanks(source, last_line_start);
    let length = run_length(source, start, fence_byte);
    let only_fence = source[start + length..body.end].trim().is_empty();
    (length >= 3 && only_fence).then_some(start..start + length)
}

pub struct LinkParts {
    pub marks: Vec<Range<usize>>,
    pub url: Option<Range<usize>>,
}

pub fn link_parts(
    source: &str,
    range: &Range<usize>,
    open_length: usize,
    text_end: usize,
) -> LinkParts {
    let mut marks = Vec::with_capacity(4);
    marks.push(range.start..range.start + open_length);
    let mut url = None;
    let tail = source.get(text_end..range.end).unwrap_or_default();
    if tail.starts_with("](") && tail.ends_with(')') {
        let inner = &tail[2..tail.len() - 1];
        let url_start = text_end + 2 + (inner.len() - inner.trim_start().len());
        let url_end = text_end + 2 + inner.trim_end().len();
        marks.extend([
            text_end..text_end + 1,
            text_end + 1..text_end + 2,
            range.end - 1..range.end,
        ]);
        url = (url_start < url_end).then_some(url_start..url_end);
    } else if tail.starts_with(']') {
        marks.push(text_end..range.end);
    }
    LinkParts { marks, url }
}

pub fn line_after(source: &str, position: usize) -> Option<Range<usize>> {
    let start = if source[..position].ends_with('\n') {
        position
    } else {
        position + source[position..].find('\n')? + 1
    };
    let end = source[start..]
        .find('\n')
        .map_or(source.len(), |newline| start + newline);
    (start < end).then_some(start..end)
}
