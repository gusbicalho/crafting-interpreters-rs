use std::io::{self, BufWriter};

mod opcode;
pub use opcode::OpCode;

mod constant;
pub use constant::Constant;

mod source_map;
pub use source_map::{LineInfo, SourceMap};

pub struct Chunk<'s> {
    constants: Vec<Constant>,
    code: Vec<u8>,
    source_map: source_map::SourceMap<'s>,
}

impl<'s> Chunk<'s> {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            code: Vec::new(),
            source_map: SourceMap::new(),
        }
    }

    pub fn push_op_code(&mut self, op: OpCode, line_info: Option<LineInfo<'s>>) {
        self.code.push(op as u8);
        if let Some(line_info) = line_info {
            self.source_map
                .set_line_info(self.code.len() - 1, line_info);
        }
    }

    pub fn push_op_arg(&mut self, arg: u8, line_info: Option<LineInfo<'s>>) {
        self.code.push(arg);
        if let Some(line_info) = line_info {
            self.source_map
                .set_line_info(self.code.len() - 1, line_info);
        }
    }

    pub fn push_constant(&mut self, constant: Constant) -> u8 {
        self.constants.push(constant);
        u8::try_from(self.constants.len() - 1).expect("Too many constants!")
    }

    pub fn describe<W>(&self, w: &mut W)
    where
        W: io::Write,
    {
        let write_line_prefix = |w: &mut W, offset: usize, previous_line: Option<usize>| -> Option<usize> {
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
        };
        let describe_simple = |w: &mut W, op: &OpCode| {
            writeln!(w, "{:?}", op).unwrap();
        };
        let describe_constant = |w: &mut W, op: &OpCode, arg: Option<u8>| match arg {
            None => {
                writeln!(w, "{:?} <MISSING INDEX>", op).unwrap();
            }
            Some(index) => match self.constants.get(usize::from(index)) {
                None => {
                    writeln!(w, "{:?} {:>4} <BAD INDEX>", op, index).unwrap();
                }
                Some(constant_value) => {
                    writeln!(w, "{:?} {:>4} {:?}", op, index, constant_value).unwrap();
                }
            },
        };
        let mut previous_line = None;
        let mut ops = self.code.iter().enumerate();
        // we will use this iterator for more stuff later
        #[allow(clippy::while_let_on_iterator)]
        while let Some((offset, op)) = ops.next() {
            previous_line = write_line_prefix(w, offset, previous_line);
            match OpCode::try_from(*op) {
                Ok(op @ OpCode::Constant) => describe_constant(w, &op, ops.next().map(|v| *v.1)),
                Ok(op) => describe_simple(w, &op),
                Err(byte) => {
                    writeln!(w, "Unknown op {}", byte).unwrap();
                }
            }
        }
    }

    pub fn describe_to_stderr(&self) {
        self.describe(&mut io::stderr().lock());
    }

    pub fn describe_to_string(&self) -> String {
        let mut buf = BufWriter::new(Vec::new());
        self.describe(&mut buf);
        String::from_utf8(buf.into_inner().unwrap()).unwrap()
    }

    pub fn get_line_info(&self, instruction_index: usize) -> Option<&LineInfo<'s>> {
        self.source_map.get_line_info(instruction_index)
    }
}

impl<'s> Default for Chunk<'s> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe() {
        let info = |line, col| Some(LineInfo::new("test", line, col));
        let mut chunk = Chunk::new();
        chunk.push_op_code(OpCode::Return, info(1, 1));
        chunk.push_op_code(OpCode::Return, None);
        let constant_index = chunk.push_constant(Constant::Number(42.0));
        chunk.push_op_code(OpCode::Constant, info(2, 3));
        chunk.push_op_arg(constant_index, info(2, 3));
        chunk.push_op_code(OpCode::Return, info(2, 4));
        assert_eq!(
            chunk.describe_to_string(),
            "0000    1 Return\n\
             0001    ? Return\n\
             0002    2 Constant    0 Number(42.0)\n\
             0004    | Return\n\
             "
        );
    }
}
