use bstr::ByteSlice;

use crate::error::EscStrError;

/// Decode a string with basic escapes
pub fn decode_str(s: &str) -> Result<String, (usize, usize, EscStrError)> {
    let mut it = s.split('\\');
    let mut ret = String::with_capacity(s.len());
    let mut skip_next = false;

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
                let hex_str = chunk.get(1..3).ok_or_else(|| {
                    (
                        chunk.as_ptr() as usize - s.as_ptr() as usize,
                        chunk.len() + 1,
                        EscStrError::TruncatedHex,
                    )
                })?;
                let v = u8::from_str_radix(hex_str, 16).unwrap_or_else(|_| todo!("{chunk:?}"));
                ret.push(char::from(v));
                ret.push_str(&chunk[3..]);
            }
            Some(b'u') => {
                if let Some(chunk) = chunk.strip_prefix("u{") {
                    if let Some(len) = chunk.as_bytes().find_not_byteset("0123456789abcdefABCDEF") {
                        let (unicode_str, chunk) = chunk.split_at(len);
                        if let Some(chunk) = chunk.strip_prefix("}") {
                            let v = u32::from_str_radix(unicode_str, 16)
                                .unwrap_or_else(|_| todo!("{chunk:?}"));
                            ret.push(char::try_from(v).unwrap_or_else(|_| todo!("{chunk:?}")));
                            ret.push_str(chunk);
                        } else {
                            todo!("{chunk:?}")
                        }
                    } else {
                        todo!("{chunk:?}")
                    }
                } else {
                    todo!("{chunk:?}")
                }
            }
            Some(&unk) => todo!("{:?}", char::from(unk)),
            None => {
                ret.push('\\');
                skip_next = true;
            }
        }
    }

    Ok(ret)
}
