use bstr::ByteSlice;

/// Decode a string with basic escapes
pub fn decode_str(s: &str) -> String {
    let mut it = s.split('\\');
    let mut ret = String::with_capacity(s.len());
    let mut skip_next = false;

    ret.push_str(it.next().unwrap_or_else(|| todo!("unreachable perhaps?")));

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
                let hex_str = chunk.get(1..3).unwrap_or_else(|| todo!("{chunk:?}"));
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

    ret
}
