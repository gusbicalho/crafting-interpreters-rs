mod opcode;
pub use opcode::OpCode;

mod source_map;
pub use source_map::{LineInfo, SourceMap};

mod bytes;
use bytes::ToBytes;

pub mod debug;

use super::value::RTValue;

pub struct Chunk<'s> {
    constants: Vec<RTValue>,
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

    pub fn get_line_info(&self, instruction_index: usize) -> Option<&LineInfo<'s>> {
        self.source_map.get_line_info(instruction_index)
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

    pub fn push_constant(&mut self, value: RTValue) -> u16 {
        self.constants.push(value);
        u16::try_from(self.constants.len() - 1).expect("Too many constants!")
    }

    pub fn push_load_constant_op(&mut self, constant_index: u16, line_info: Option<LineInfo<'s>>) {
        match u8::try_from(constant_index) {
            Ok(byte) => {
                self.push_op_code(OpCode::Constant, line_info.clone());
                self.push_op_arg(byte, line_info);
            }
            Err(_) => {
                self.push_op_code(OpCode::ConstantLong, line_info.clone());
                for byte in ToBytes::<2>::num_to_bytes(&constant_index) {
                    self.push_op_arg(byte, line_info.clone());
                }
            }
        }
    }

    pub fn push_constant_and_load_op(&mut self, value: RTValue, line_info: Option<LineInfo<'s>>) {
        let constant_index = self.push_constant(value);
        self.push_load_constant_op(constant_index, line_info);
    }
}

impl<'s> Default for Chunk<'s> {
    fn default() -> Self {
        Self::new()
    }
}
