use std::{env, io};

use crate::pipeline::{self, scanner};

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
    if let Some(path) = env::args().nth(1) {
        interpret(pipeline::source::from_file(&path).expect("Failed to open file"));
    }

    loop {
        let input: String = io::stdin()
            .lines()
            .map_while(|line| line.ok().filter(|line| !line.is_empty()))
            .intersperse("\n".to_string())
            .collect();
        if input.is_empty() {
            break;
        }
        interpret(pipeline::source::from_repl_input(&input));
    }
}
