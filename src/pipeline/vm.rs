use super::{
    bytecode::{
        self,
        bytes::{self, FromBytes},
        Chunk, OpCode,
    },
    value::RTValue,
};

pub const STACK_MAX: usize = 65535;

pub struct VM<'s> {
    chunk: Option<&'s Chunk<'s>>,
    stack: Vec<RTValue>,
}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError(String),
}

macro_rules! unwrap_or_bail {
    ($option:expr) => {
        match $option {
            Some(payload) => payload,
            None => {
                return Ok(());
            }
        }
    };
    ($option:expr, $err:expr) => {
        match $option {
            Some(payload) => payload,
            None => {
                return Err($err);
            }
        }
    };
}

macro_rules! ok_or_bail_with {
    ($result:expr, $mkerr:expr) => {
        match $result {
            Ok(payload) => payload,
            Err(err) => {
                return Err($mkerr(err));
            }
        }
    };
}

macro_rules! debug_run {
    ($arg:block) => {
        if cfg!(debug_assertions) {
            $arg
        }
    };
}

macro_rules! read_constant {
    ($chunk: expr, $ip: expr, $bytes:expr, $error_msg: expr) => {{
        let constant_index = unwrap_or_bail!(
            bytes::next_bytes::<$bytes>($ip),
            InterpretError::RuntimeError($error_msg.to_string())
        )
        .bytes_to_num();
        *unwrap_or_bail!(
            $chunk.get_constant(constant_index),
            InterpretError::RuntimeError(format!("Bad constant index {}", constant_index))
        )
    }};
}

impl<'s> VM<'s> {
    pub fn new() -> Self {
        let mut stack = Vec::new();
        stack.reserve(STACK_MAX);
        Self { chunk: None, stack }
    }

    pub fn with_chunk<'t>(mut self, new_chunk: &'t Chunk<'t>) -> VM<'t> {
        self.stack.clear();
        VM {
            chunk: Some(new_chunk),
            ..self
        }
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        let chunk = unwrap_or_bail!(self.chunk);
        let mut ip = chunk.code().iter();
        loop {
            let opcode = unwrap_or_bail!(ip.next());
            debug_run!({
                eprintln!("{:#?}", self.stack);
                {
                    let mut ip = ip.clone();
                    let offset: usize =
                        opcode as *const u8 as usize - chunk.code().as_ptr() as usize;
                    chunk.describe_instruction_to_stderr(offset, *opcode, &mut ip);
                }
            });
            let opcode = ok_or_bail_with!(OpCode::try_from(*opcode), |byte| {
                InterpretError::RuntimeError(format!("Expected OpCode, got byte {}", byte))
            });
            match opcode {
                OpCode::Return => {
                    println!("{:?}", self.stack.pop());
                    return Ok(());
                }
                OpCode::Constant => {
                    self.stack.push(read_constant!(
                        chunk,
                        &mut ip,
                        1,
                        "Missing arg byte for Constant"
                    ));
                }
                OpCode::ConstantLong => {
                    self.stack.push(read_constant!(
                        chunk,
                        &mut ip,
                        { bytecode::CONSTANT_LONG_ARG_BYTES },
                        "Missing arg bytes for ConstantLong"
                    ));
                }
            }
        }
    }
}

impl<'s> Default for VM<'s> {
    fn default() -> Self {
        Self::new()
    }
}
