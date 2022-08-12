mod opcode;
use std::mem;

pub use opcode::OpCode;

mod source_map;
pub use source_map::{LineInfo, SourceMap};

pub mod bytes;
use bytes::ToBytes;

pub mod debug;

use super::value::RTValue;

pub struct Chunk<'s> {
    constants: Vec<RTValue>,
    code: Vec<u8>,
    source_map: source_map::SourceMap<'s>,
}

pub type ConstantIndex = usize;
pub const CONSTANT_LONG_ARG_BYTES: usize = mem::size_of::<ConstantIndex>();

impl<'s> Chunk<'s> {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            code: Vec::new(),
            source_map: SourceMap::new(),
        }
    }

    pub fn code(&self) -> &[u8] {
        self.code.as_ref()
    }

    #[inline(always)]
    pub fn get_constant(&self, constant_index: ConstantIndex) -> Option<&RTValue> {
        self.constants.get(constant_index)
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

    pub fn push_constant(&mut self, value: RTValue) -> ConstantIndex {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn push_load_constant_op(
        &mut self,
        constant_index: ConstantIndex,
        line_info: Option<LineInfo<'s>>,
    ) {
        match u8::try_from(constant_index) {
            Ok(byte) => {
                self.push_op_code(OpCode::Constant, line_info.clone());
                self.push_op_arg(byte, line_info);
            }
            Err(_) => {
                self.push_op_code(OpCode::ConstantLong, line_info.clone());
                for byte in ToBytes::<CONSTANT_LONG_ARG_BYTES>::num_to_bytes(&constant_index) {
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
