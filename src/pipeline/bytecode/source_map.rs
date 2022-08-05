use rangemap::RangeMap;

pub struct SourceMap<'s> {
    line_info: RangeMap<usize, LineInfo<'s>>,
}

impl<'s> SourceMap<'s> {
    pub fn new() -> Self {
        Self {
            line_info: RangeMap::new(),
        }
    }

    pub fn set_line_info(&mut self, instruction_index: usize, info: LineInfo<'s>) {
        self.line_info.insert(instruction_index..instruction_index+1, info);
    }

    pub fn get_line_info(&self, instruction_index: usize) -> Option<&LineInfo<'s>> {
        self.line_info.get(&instruction_index)
    }
}

impl<'s> Default for SourceMap<'s> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineInfo<'s> {
    pub source_name: &'s str,
    pub line: usize,
    pub column: usize,
}

impl<'s> LineInfo<'s> {
    pub fn new(source_name: &'s str, line: usize, column: usize) -> Self {
        Self {
            source_name,
            line,
            column,
        }
    }
}
