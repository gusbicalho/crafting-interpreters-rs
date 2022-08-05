use std::{fs, io};

pub struct Source {
    name: String,
    text: String,
}

impl Source {
    pub fn new(name: String, text: String) -> Self {
        Self { name, text }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

pub fn from_file(path: &str) -> io::Result<Source> {
    Ok(Source::new(path.to_string(), fs::read_to_string(path)?))
}

pub fn from_repl_input(input: &str) -> Source {
    Source::new("REPL".to_string(), input.to_string())
}
