use crate::{
    error::Result,
    token::Brace,
    type_trait::{LogixTypeDescriptor, LogixValueDescriptor, Value},
    LogixParser, LogixType,
};

impl<const SIZE: usize, T: LogixType> LogixType for [T; SIZE] {
    fn descriptor() -> &'static LogixTypeDescriptor {
        &LogixTypeDescriptor {
            name: "Array",
            doc: "a fixed size array",
            value: LogixValueDescriptor::Native,
        }
    }

    fn default_value() -> Option<Self> {
        None
    }

    fn logix_parse<FS: logix_vfs::LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        p.req_wrapped("Array", Brace::Square, |p| {
            let mut ret = Vec::with_capacity(SIZE); // TODO(2023.04.01): Change to array.try_map when stable (rust-lang#79711)
            let mut it = p.parse_delimited::<T>("Array");
            for _ in 0..SIZE {
                ret.push(it.req_next_item()?.value);
            }
            it.req_end(Brace::Square)?;
            Ok(ret.try_into().ok().unwrap())
        })
    }
}

impl<T: LogixType> LogixType for Vec<T> {
    fn descriptor() -> &'static LogixTypeDescriptor {
        &LogixTypeDescriptor {
            name: "list",
            doc: "a dynamically sized array",
            value: LogixValueDescriptor::Native,
        }
    }

    fn default_value() -> Option<Self> {
        None
    }

    fn logix_parse<FS: logix_vfs::LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        p.req_wrapped("list", Brace::Square, |p| {
            let mut ret = Vec::new();
            for item in p.parse_delimited::<T>("list") {
                let Value { value, span: _ } = item?;
                ret.push(value);
            }
            Ok(ret)
        })
    }
}
