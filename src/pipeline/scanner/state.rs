use std::mem;

pub struct ScanState<'s> {
    source: &'s str,
    segment: String,
    start_pos: Position,
    current_pos: Position,
}

impl<'s> ScanState<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            segment: String::new(),
            start_pos: Position::zero(),
            current_pos: Position::zero(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl<'s> ScanState<'s> {
    pub fn is_at_end(&self) -> bool {
        self.source.is_empty()
    }

    pub fn current_position(&self) -> &Position {
        &self.current_pos
    }

    pub fn segment(&self) -> (&str, &Position, &Position) {
        (&self.segment, &self.start_pos, &self.current_pos)
    }

    pub fn take_segment(&mut self) -> (String, Position, Position) {
        let segment = mem::take(&mut self.segment);
        self.start_pos = self.current_pos;
        (segment, self.start_pos, self.current_pos)
    }

    pub fn reset_segment(&mut self) {
        self.segment.clear();
        self.start_pos = self.current_pos;
    }

    pub fn advance(&mut self, distance: usize) -> &'s str {
        let (consumed, leftovers) = self.source.split_at(distance);
        self.source = leftovers;
        let mut newlines = consumed.rmatch_indices('\n');
        match newlines.next() {
            None => self.current_pos.column += consumed.len(),
            Some((index_of_last_newline, _)) => {
                self.current_pos.column = consumed.len() - index_of_last_newline - 1;
                self.current_pos.line += newlines.count() + 1;
            }
        }
        self.segment.push_str(consumed);
        consumed
    }

    pub fn pop_char(&mut self) -> Option<char> {
        self.source.chars().next().map(|c| {
            self.advance(c.len_utf8());
            c
        })
    }

    pub fn match_pred<F>(&mut self, pred: F) -> Option<char>
    where
        F: FnOnce(char) -> bool,
    {
        self.source.chars().next().filter(|c| pred(*c)).map(|c| {
            self.advance(c.len_utf8());
            c
        })
    }

    pub fn match_char(&mut self, expected: char) -> bool {
        self.match_pred(|c| c == expected).is_some()
    }

    pub fn peek_char(&mut self) -> Option<char> {
        self.source.chars().next()
    }

    pub fn peek_chars<const N: usize>(&mut self) -> Option<[char; N]> {
        let mut found_chars: usize = 0;
        let mut cs: [char; N] = ['\0'; N];
        for (i, c) in self.source.chars().take(N).enumerate() {
            found_chars += 1;
            cs[i] = c;
        }
        if found_chars == N {
            Some(cs)
        } else {
            None
        }
    }
}
