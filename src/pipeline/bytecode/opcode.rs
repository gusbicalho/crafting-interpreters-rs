use std::mem;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Return = 0,
    Constant = 1,
    ConstantLong = 2,
}
const MAX_OPCODE: u8 = 2;

impl TryFrom<u8> for OpCode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MAX_OPCODE {
            Err(value)
        } else {
            Ok(unsafe { mem::transmute(value) })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::OpCode;
    #[test]
    fn opcode_size() {
        assert_eq!(mem::size_of::<OpCode>(), 1)
    }
}
