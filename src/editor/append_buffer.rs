pub struct AppendBuffer {
    pub buffer: Vec<u8>,
}

impl AppendBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn append(&mut self, s: &str) {
        self.buffer.extend(s.bytes());
    }

    pub fn free(&mut self) {
        self.buffer.clear();
    }
}

