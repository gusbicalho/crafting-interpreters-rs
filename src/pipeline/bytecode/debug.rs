use std::io::{self, BufWriter, Write};

use super::{
    bytes::{self, FromBytes},
    Chunk, ConstantIndex, OpCode, CONSTANT_LONG_ARG_BYTES,
};

impl<'s> Chunk<'s> {
    fn write_line_prefix<W: io::Write>(&self, w: &mut W, offset: usize) {
        write!(w, "{:0>4} ", offset).unwrap();
        let previous_line = offset
            .checked_sub(1)
            .and_then(|offset| self.source_map.get_line_info(offset).map(|li| li.line));
        match self.source_map.get_line_info(offset) {
            Some(line_info) => {
                if previous_line == Some(line_info.line) {
                    write!(w, "   | ").unwrap();
                } else {
                    write!(w, "{:>4} ", line_info.line).unwrap();
                }
            }
            None => {
                write!(w, "   ? ").unwrap();
            }
        }
    }

    fn describe_simple<W: io::Write>(w: &mut W, op: &OpCode) {
        writeln!(w, "{:?}", op).unwrap();
    }

    fn describe_constant<const N: usize, W: io::Write>(
        &self,
        w: &mut W,
        op: &OpCode,
        arg_bytes: [Option<u8>; N],
    ) where
        [u8; N]: FromBytes<ConstantIndex>,
    {
        match bytes::all_there(&arg_bytes).map(|bytes| bytes.bytes_to_num()) {
            None => {
                writeln!(
                    w,
                    "{:?} <BAD BYTES>{:?}",
                    op,
                    arg_bytes
                        .iter()
                        .map_while(|opt_byte| *opt_byte)
                        .collect::<Vec<u8>>()
                )
                .unwrap();
            }
            Some(index) => match self.get_constant(index) {
                None => {
                    writeln!(w, "{:?} {:>4} <BAD INDEX>", op, index).unwrap();
                }
                Some(constant_value) => {
                    writeln!(w, "{:?} {:>4} {:?}", op, index, constant_value).unwrap();
                }
            },
        }
    }

    pub fn describe_instruction<'i, W>(
        &self,
        w: &mut W,
        offset: usize,
        op_byte: u8,
        ops: &mut impl Iterator<Item = &'i u8>,
    ) where
        W: io::Write,
    {
        self.write_line_prefix(w, offset);
        match OpCode::try_from(op_byte) {
            Ok(op @ OpCode::Constant) => {
                self.describe_constant(w, &op, bytes::try_next_bytes::<1, _, _>(ops, |b| *b))
            }
            Ok(op @ OpCode::ConstantLong) => self.describe_constant(
                w,
                &op,
                bytes::try_next_bytes::<CONSTANT_LONG_ARG_BYTES, _, _>(ops, |b| *b),
            ),
            Ok(op) => Self::describe_simple(w, &op),
            Err(byte) => {
                writeln!(w, "Unknown op {}", byte).unwrap();
            }
        }
    }

    pub fn describe<W>(&self, w: &mut W)
    where
        W: io::Write,
    {
        let mut ops = DebugIter::new(self.code.iter());
        while let Some(op) = ops.next() {
            self.describe_instruction(w, ops.offset() - 1, *op, &mut ops);
        }
    }

    pub fn describe_instruction_to_stderr<'i>(
        &self,
        offset: usize,
        op_byte: u8,
        ops: &mut impl Iterator<Item = &'i u8>,
    ) {
        self.describe_instruction(&mut io::stderr().lock(), offset, op_byte, ops)
    }

    pub fn describe_to_stderr(&self, chunk_name: Option<&str>) {
        let mut out = io::stderr().lock();
        match chunk_name {
            Some(chunk_name) => {
                writeln!(out, "== {} ==", chunk_name).unwrap();
            }
            None => {
                writeln!(out, "========").unwrap();
            }
        }
        self.describe(&mut out);
        writeln!(out, "========").unwrap();
    }

    pub fn describe_to_string(&self) -> String {
        let mut buf = BufWriter::new(Vec::new());
        self.describe(&mut buf);
        String::from_utf8(buf.into_inner().unwrap()).unwrap()
    }
}

struct DebugIter<I> {
    inner: I,
    offset: usize,
}

impl<I> DebugIter<I> {
    fn new(inner: I) -> Self {
        Self { inner, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }
}

impl<I> Iterator for DebugIter<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|v| {
            self.offset += 1;
            v
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline::value::RTValue;

    use super::{super::LineInfo, *};

    #[test]
    fn test_describe() {
        let info = |line, col| Some(LineInfo::new("test", line, col));
        let mut chunk = Chunk::new();
        chunk.push_op_code(OpCode::Return, info(1, 1));
        chunk.push_op_code(OpCode::Return, None);
        let constant_index = chunk.push_constant(RTValue::Number(42.0));
        chunk.push_load_constant_op(constant_index, info(2, 3));
        chunk.push_op_code(OpCode::Return, info(2, 4));
        chunk.push_load_constant_op(300, info(3, 7));
        chunk.push_op_code(OpCode::ConstantLong, info(7, 4));
        chunk.push_op_arg(7, info(7, 4));
        assert_eq!(
            chunk.describe_to_string(),
            "0000    1 Return\n\
             0001    ? Return\n\
             0002    2 Constant    0 Number(42.0)\n\
             0004    | Return\n\
             0005    3 ConstantLong  300 <BAD INDEX>\n\
             0014    7 ConstantLong <BAD BYTES>[7]\n\
             "
        );
    }
}
