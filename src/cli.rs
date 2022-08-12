use std::{env, io};

use crate::pipeline::{
    self,
    bytecode::{Chunk, LineInfo, OpCode},
    scanner,
    value::RTValue, vm::VM,
};

pub struct CliConfig {}

impl CliConfig {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CliConfig {
        Self {}
    }
}

fn interpret(source: pipeline::source::Source) {
    let tokens = scanner::scan(source);
    println!("{:?}", tokens);
}

pub fn run(_config: &CliConfig) {
    // if let Some(path) = env::args().nth(1) {
    //     interpret(pipeline::source::from_file(&path).expect("Failed to open file"));
    // }

    // loop {
    //     let input: String = io::stdin()
    //         .lines()
    //         .map_while(|line| line.ok().filter(|line| !line.is_empty()))
    //         .intersperse("\n".to_string())
    //         .collect();
    //     if input.is_empty() {
    //         break;
    //     }
    //     interpret(pipeline::source::from_repl_input(&input));
    // }

    let mut vm = VM::new();
    let source = "source";
    let mut chunk = Chunk::new();
    chunk.push_constant_and_load_op(RTValue::Number(1.2), Some(LineInfo::new(source, 123, 0)));
    chunk.push_op_code(OpCode::Return, Some(LineInfo::new(source, 123, 1)));
    chunk.describe_to_stderr(Some("test chunk"));
    vm.with_chunk(&chunk).run().unwrap();
}
