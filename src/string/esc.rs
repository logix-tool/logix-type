use crate::error::EscStrError;

/// Decode a string with basic escapes
pub fn decode_str(s: &str) -> Result<String, (usize, usize, EscStrError)> {
    let mut it = s.split('\\');
    let mut ret = String::with_capacity(s.len());
    let mut skip_next = false;

    let get_off = |c: &str| c.as_ptr() as usize - s.as_ptr() as usize;

    ret.push_str(it.next().unwrap()); // First chunk is always valid, and always exist

    for chunk in it {
        if std::mem::take(&mut skip_next) {
            ret.push_str(chunk);
            continue;
        }

        match chunk.as_bytes().first() {
            Some(b'r') => {
                ret.push('\r');
                ret.push_str(&chunk[1..]);
            }
            Some(b'n') => {
                ret.push('\n');
                ret.push_str(&chunk[1..]);
            }
            Some(b't') => {
                ret.push('\t');
                ret.push_str(&chunk[1..]);
            }
            Some(b'"') => {
                ret.push('"');
                ret.push_str(&chunk[1..]);
            }
            Some(b'x') => {
                let hex_str = chunk
                    .get(1..3)
                    .ok_or_else(|| (get_off(chunk), chunk.len() + 1, EscStrError::TruncatedHex))?;
                let v = u8::from_str_radix(hex_str, 16).map_err(|_| {
                    (
                        get_off(hex_str) - 1,
                        hex_str.len() + 2,
                        EscStrError::InvalidHex,
                    )
                })?;
                ret.push(char::from(v));
                ret.push_str(&chunk[3..]);
            }
            Some(b'u') => {
                let chunk = chunk.strip_prefix("u{").ok_or_else(|| {
                    (
                        get_off(chunk),
                        3,
                        EscStrError::InvalidUnicodeMissingStartBrace,
                    )
                })?;

                let len = chunk
                    .chars()
                    .take(9)
                    .position(|c| matches!(c, '}'))
                    .ok_or_else(|| {
                        (
                            get_off(chunk) - 2,
                            chunk.len().min(8) + 3,
                            EscStrError::InvalidUnicodeMissingEndBrace,
                        )
                    })?;

                let (unicode_str, chunk) = chunk.split_at(len);
                let v = u32::from_str_radix(unicode_str, 16).map_err(|_| {
                    (
                        get_off(unicode_str) - 2,
                        unicode_str.len() + 4,
                        EscStrError::InvalidUnicodeHex,
                    )
                })?;
                ret.push(char::try_from(v).map_err(|_| {
                    (
                        get_off(unicode_str) - 2,
                        unicode_str.len() + 4,
                        EscStrError::InvalidUnicodePoint(v),
                    )
                })?);
                ret.push_str(&chunk[1..]);
            }
            Some(&unk) => {
                return Err((
                    get_off(chunk),
                    2,
                    EscStrError::InvalidEscapeChar(unk.into()),
                ))
            }
            None => {
                ret.push('\\');
                skip_next = true;
            }
        }
    }

    Ok(ret)
}
