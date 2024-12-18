mod append_buffer;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    mem::take,
    process::exit,
};

use append_buffer::AppendBuffer;

mod terminal;
use terminal::{
    Terminal, CURSOR_POSITION_START, ERASE_LINE, HIDE_CURSOR, SHOW_CURSOR,
};

const KILO_VERSION: &str = "0.0.4";

#[repr(i32)]
enum EditorKey {
    ArrowLeft = 1000,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
}

pub struct Editor {
    append_buffer: AppendBuffer,
    cursor_col: i32,
    cursor_row: i32,
    line: String,
    num_rows: i32,
    screen_cols: i32,
    screen_rows: i32,
    terminal: Terminal,
}

impl Editor {
    fn draw_rows(&mut self) -> () {
        for i in 0..self.screen_rows {
            if i >= self.num_rows {
                if i == self.screen_rows / 3 {
                    let message =
                        format!("Kilo editor -- version {}", KILO_VERSION);
                    let message_length = message.len();
                    if message_length == self.screen_cols as usize {
                        self.append_buffer
                            .append(&message[..self.screen_cols as usize]);
                    } else {
                        let padding =
                            (self.screen_cols as usize - message_length) / 2;
                        let padded_message = format!(
                            "{:<padding$}{message:padding$}",
                            "",
                            message = message,
                            padding = padding
                        );
                        self.append_buffer.append(&padded_message);
                    }
                } else {
                    self.append_buffer.append("~");
                }
            } else {
                let length = if self.line.len() > self.screen_cols as usize {
                    self.screen_cols as usize
                } else {
                    self.line.len()
                };
                self.append_buffer.append(&self.line[..length]);
            }
            self.append_buffer.append(ERASE_LINE);
            if i < self.screen_rows - 1 {
                self.append_buffer.append("\r\n");
            }
        }
    }

    fn exit(&mut self) -> Result<(), Error> {
        self.refresh_screen()?;
        self.terminal.disable_raw_mode()?;
        exit(0);
    }

    fn move_cursor(&mut self, key: EditorKey) -> () {
        match key {
            EditorKey::ArrowLeft => {
                self.cursor_col = self.cursor_col.saturating_sub(1)
            }
            EditorKey::ArrowRight => {
                if self.cursor_col < self.screen_cols - 1 {
                    self.cursor_col = self.cursor_col.saturating_add(1)
                }
            }
            EditorKey::ArrowUp => {
                self.cursor_row = self.cursor_row.saturating_sub(1)
            }
            EditorKey::ArrowDown => {
                if self.cursor_row < self.screen_rows - 1 {
                    self.cursor_row = self.cursor_row.saturating_add(1)
                }
            }
            EditorKey::Home => self.cursor_col = 0,
            EditorKey::End => self.cursor_col = self.screen_cols - 1,
            EditorKey::PageUp => {
                while self.cursor_row > 0 {
                    self.cursor_row = self.cursor_row.saturating_sub(1);
                }
            }
            EditorKey::PageDown => {
                while self.cursor_row < self.screen_rows - 1 {
                    self.cursor_row = self.cursor_row.saturating_add(1);
                }
            }
            _ => {}
        }
    }

    fn open(&mut self, file_name: &str) -> Result<(), Error> {
        let file = File::open(file_name)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        self.line = if line.ends_with('\n') {
            match line.pop() {
                Some('\n') => line,
                _ => line,
            }
        } else {
            line.to_string()
        };
        Ok(self.num_rows = 1)
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let key = self.read_key()?;
        match key {
            xon if xon == b'\x11' as i32 => self.exit(),
            arrow_left if arrow_left == EditorKey::ArrowLeft as i32 => {
                Ok(self.move_cursor(EditorKey::ArrowLeft))
            }
            arrow_right if arrow_right == EditorKey::ArrowRight as i32 => {
                Ok(self.move_cursor(EditorKey::ArrowRight))
            }
            arrow_up if arrow_up == EditorKey::ArrowUp as i32 => {
                Ok(self.move_cursor(EditorKey::ArrowUp))
            }
            arrow_down if arrow_down == EditorKey::ArrowDown as i32 => {
                Ok(self.move_cursor(EditorKey::ArrowDown))
            }
            home if home == EditorKey::Home as i32 => {
                Ok(self.move_cursor(EditorKey::Home))
            }
            end if end == EditorKey::End as i32 => {
                Ok(self.move_cursor(EditorKey::End))
            }
            page_up if page_up == EditorKey::PageUp as i32 => {
                Ok(self.move_cursor(EditorKey::PageUp))
            }
            page_down if page_down == EditorKey::PageDown as i32 => {
                Ok(self.move_cursor(EditorKey::PageDown))
            }
            _ => Ok(()),
        }
    }

    fn read_key(&mut self) -> Result<i32, Error> {
        let mut sequence = [0; 4];
        sequence[0] = self.terminal.read_single_byte_from_input()?;
        if sequence[0] == b'\x1b' {
            sequence[1] = self.terminal.read_single_byte_from_input()?;
            sequence[2] = self.terminal.read_single_byte_from_input()?;
            sequence[3] = self.terminal.read_single_byte_from_input()?;
            if sequence[1] == b'[' {
                if sequence[2] >= b'0' && sequence[2] <= b'9' {
                    if sequence[3] == b'~' {
                        match sequence[2] {
                            b'1' => return Ok(EditorKey::Home as i32),
                            b'3' => return Ok(EditorKey::Delete as i32),
                            b'4' => return Ok(EditorKey::End as i32),
                            b'5' => return Ok(EditorKey::PageUp as i32),
                            b'6' => return Ok(EditorKey::PageDown as i32),
                            b'7' => return Ok(EditorKey::Home as i32),
                            b'8' => return Ok(EditorKey::End as i32),
                            _ => return Ok(sequence[0] as i32),
                        }
                    }
                }
                match sequence[2] {
                    b'A' => return Ok(EditorKey::ArrowUp as i32),
                    b'B' => return Ok(EditorKey::ArrowDown as i32),
                    b'C' => return Ok(EditorKey::ArrowRight as i32),
                    b'D' => return Ok(EditorKey::ArrowLeft as i32),
                    b'H' => return Ok(EditorKey::Home as i32),
                    b'F' => return Ok(EditorKey::End as i32),
                    _ => return Ok(sequence[0] as i32),
                }
            } else if sequence[1] == b'O' {
                match sequence[2] {
                    b'H' => return Ok(EditorKey::Home as i32),
                    b'F' => return Ok(EditorKey::End as i32),
                    _ => return Ok(sequence[0] as i32),
                }
            }
        }
        Ok(sequence[0] as i32)
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        self.append_buffer.append(HIDE_CURSOR);
        self.append_buffer.append(CURSOR_POSITION_START);
        self.draw_rows();
        Terminal::set_cursor_position_buffer(
            self.cursor_row + 1,
            self.cursor_col + 1,
            &mut self.append_buffer.chars,
        );
        self.append_buffer.append(SHOW_CURSOR);
        let buffer = take(&mut self.append_buffer.chars);
        self.terminal.write_output_from_buffer(buffer)
    }

    pub fn run(&mut self, file_name: Option<&str>) -> Result<(), Error> {
        if let Some(file_to_open) = file_name {
            self.open(file_to_open)?;
        }
        loop {
            self.refresh_screen()?;
            self.process_keypress()?;
        }
    }

    pub fn try_new() -> Result<Self, Error> {
        let mut terminal = Terminal::try_new()?;
        terminal.enable_raw_mode()?;
        let (rows, cols) = terminal.get_window_size()?;
        let editor = Self {
            append_buffer: AppendBuffer::new(),
            cursor_col: 0,
            cursor_row: 0,
            line: String::new(),
            num_rows: 0,
            screen_cols: cols,
            screen_rows: rows,
            terminal,
        };
        Ok(editor)
    }
}
