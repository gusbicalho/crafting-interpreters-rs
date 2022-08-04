pub struct CliConfig {}

impl CliConfig {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CliConfig {
        Self {}
    }
}

pub fn run(_config: &CliConfig) {
    println!("hello!");
}
