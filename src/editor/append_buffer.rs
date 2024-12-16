pub struct AppendBuffer {
    pub chars: Vec<u8>,
}

impl AppendBuffer {
    pub fn new() -> Self {
        Self { chars: Vec::new() }
    }

    pub fn append(&mut self, s: &str) {
        self.chars.extend(s.bytes());
    }
}
