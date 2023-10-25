use bstr::ByteSlice;

/// Decode a string in the txt format, see StrTag::Txt
pub fn decode_str(s: &str) -> String {
    let mut lines = Vec::new();
    let mut pos = 0;
    let mut prefix_len = usize::MAX;

    for l in s.as_bytes().lines_with_terminator() {
        let raw_end = pos + l.len();
        let end = pos + l.trim_end().len();

        // Enter the if only if not the first line, or it is not a newline
        if pos != 0 || pos != end {
            if pos != end {
                prefix_len = prefix_len.min(l.len() - l.trim_start().len());
            }
            lines.push(pos..end);
        }
        pos = raw_end;
    }

    // Remove the last newline
    if let Some(true) = lines.last().map(|r| r.is_empty()) {
        lines.pop();
    }

    let mut ret = String::with_capacity(lines.iter().map(|r| r.len().max(1)).sum());

    for range in lines {
        debug_assert!(ret.capacity() >= range.len(), "{range:?}");
        let cur = &s[range];
        if cur.is_empty() {
            ret.push('\n');
        } else {
            ret.push_str(&cur[prefix_len..]);
        }
    }

    ret
}
