#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpan {
    pub start: usize,
    pub end: usize,
}

impl LineSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        let mut line = 1usize;
        let mut start_byte = 0usize;
        let mut end_byte = src.len();
        for (i, b) in src.bytes().enumerate() {
            if line == self.start && start_byte == 0 {
                start_byte = i;
            }
            if line == self.end + 1 {
                end_byte = i;
                break;
            }
            if b == b'\n' {
                line += 1;
            }
        }
        &src[start_byte..end_byte]
    }
}
