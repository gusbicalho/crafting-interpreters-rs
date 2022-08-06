use std::io::{self, BufWriter, Write};

use super::{
    bytes::{self, FromBytes},
    Chunk, OpCode,
};

impl<'s> Chunk<'s> {
    fn write_line_prefix<W: io::Write>(
        &self,
        w: &mut W,
        offset: usize,
        previous_line: Option<usize>,
    ) -> Option<usize> {
        write!(w, "{:0>4} ", offset).unwrap();
        match self.source_map.get_line_info(offset) {
            Some(line_info) => {
                if previous_line == Some(line_info.line) {
                    write!(w, "   | ").unwrap();
                } else {
                    write!(w, "{:>4} ", line_info.line).unwrap();
                }
                Some(line_info.line)
            }
            None => {
                write!(w, "   ? ").unwrap();
                None
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
        [u8; N]: FromBytes<u16>,
    {
        match bytes::all_there(&arg_bytes).map(|bytes| bytes.bytes_to_num()) {
            None => {
                writeln!(
                    w,
                    "{:?} <BAD BYTES>{:?}",
                    op,
                    arg_bytes
                        .iter()
                        .map(|a| a
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "MISSING".to_string()))
                        .collect::<Vec<String>>()
                )
                .unwrap();
            }
            Some(index) => match self.constants.get(usize::from(index)) {
                None => {
                    writeln!(w, "{:?} {:>4} <BAD INDEX>", op, index).unwrap();
                }
                Some(constant_value) => {
                    writeln!(w, "{:?} {:>4} {:?}", op, index, constant_value).unwrap();
                }
            },
        }
    }

    pub fn describe<W>(&self, w: &mut W)
    where
        W: io::Write,
    {
        let mut previous_line = None;
        let mut ops = self.code.iter().enumerate();
        fn next_bytes<'l, const N: usize>(
            code: &mut impl Iterator<Item = (usize, &'l u8)>,
        ) -> [Option<u8>; N] {
            let mut result = [None; N];
            for i in result.iter_mut() {
                *i = code.next().map(|v| *v.1);
            }
            result
        }
        // we will use this iterator for more stuff later
        #[allow(clippy::while_let_on_iterator)]
        while let Some((offset, op)) = ops.next() {
            previous_line = self.write_line_prefix(w, offset, previous_line);
            match OpCode::try_from(*op) {
                Ok(op @ OpCode::Constant) => {
                    self.describe_constant(w, &op, next_bytes::<1>(&mut ops))
                }
                Ok(op @ OpCode::ConstantLong) => {
                    self.describe_constant(w, &op, next_bytes::<2>(&mut ops))
                }
                Ok(op) => Self::describe_simple(w, &op),
                Err(byte) => {
                    writeln!(w, "Unknown op {}", byte).unwrap();
                }
            }
        }
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
        chunk.push_load_constant_op(300u16, info(3, 7));
        chunk.push_op_code(OpCode::ConstantLong, info(7, 4));
        chunk.push_op_arg(7, info(7, 4));
        assert_eq!(
            chunk.describe_to_string(),
            "0000    1 Return\n\
             0001    ? Return\n\
             0002    2 Constant    0 Number(42.0)\n\
             0004    | Return\n\
             0005    3 ConstantLong  300 <BAD INDEX>\n\
             0008    7 ConstantLong <BAD BYTES>[\"7\", \"MISSING\"]\n\
             "
        );
    }
}
