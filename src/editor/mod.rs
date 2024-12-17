use std::io::Error;
use std::mem::take;
use std::process::exit;
use std::u16;

mod append_buffer;
use append_buffer::AppendBuffer;

mod terminal;
use terminal::{
    Terminal, CURSOR_POSITION_START, ERASE_LINE, HIDE_CURSOR, SHOW_CURSOR,
};

const KILO_VERSION: &str = "0.0.1";

#[repr(i32)]
enum EditorKey {
    ArrowLeft = 1000,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
}

pub struct Editor {
    append_buffer: AppendBuffer,
    cursor_col: u16,
    cursor_row: u16,
    screen_cols: u16,
    screen_rows: u16,
    terminal: Terminal,
}

impl Editor {
    fn draw_rows(&mut self) -> Result<(), Error> {
        for i in 0..self.screen_rows {
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
            self.append_buffer.append(ERASE_LINE);
            if i < self.screen_rows - 1 {
                self.append_buffer.append("\r\n");
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.refresh_screen().unwrap_or_else(|e| {
            eprintln!("Error refreshing screen: {}", e);
            exit(1);
        });
        self.terminal.disable_raw_mode().unwrap_or_else(|e| {
            eprintln!("Error disabling raw mode: {}", e);
            exit(1);
        });
        exit(0);
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        if let Some(key) = self.read_key()? {
            match key {
                xon if xon == b'\x11' as i32 => Ok(self.exit()),
                arrow_left if arrow_left == EditorKey::ArrowLeft as i32 => {
                    return self.move_cursor(EditorKey::ArrowLeft);
                }
                arrow_right if arrow_right == EditorKey::ArrowRight as i32 => {
                    return self.move_cursor(EditorKey::ArrowRight);
                }
                arrow_up if arrow_up == EditorKey::ArrowUp as i32 => {
                    return self.move_cursor(EditorKey::ArrowUp);
                }
                arrow_down if arrow_down == EditorKey::ArrowDown as i32 => {
                    return self.move_cursor(EditorKey::ArrowDown);
                }
                page_up if page_up == EditorKey::PageUp as i32 => {
                    return self.move_cursor(EditorKey::PageUp);
                }
                page_down if page_down == EditorKey::PageDown as i32 => {
                    return self.move_cursor(EditorKey::PageDown);
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn read_key(&mut self) -> Result<Option<i32>, Error> {
        let mut sequence = [0; 3];
        match self.terminal.read_single_byte_from_input()? {
            Some(byte) => sequence[0] = byte,
            None => return Ok(None),
        };
        if sequence[0] == b'\x1b' {
            match self.terminal.read_single_byte_from_input()? {
                Some(byte) => sequence[1] = byte,
                None => return Ok(Some(b'\x1b' as i32)),
            };
            match self.terminal.read_single_byte_from_input()? {
                Some(byte) => sequence[2] = byte,
                None => return Ok(Some(b'\x1b' as i32)),
            };
            if sequence[1] == b'[' {
                if sequence[2] >= b'0' && sequence[2] <= b'9' {
                    let fourth_byte =
                        match self.terminal.read_single_byte_from_input()? {
                            Some(byte) => byte,
                            None => return Ok(Some(b'\x1b' as i32)),
                        };
                    if fourth_byte == b'~' {
                        match sequence[2] {
                            b'5' => return Ok(Some(EditorKey::PageUp as i32)),
                            b'6' => {
                                return Ok(Some(EditorKey::PageDown as i32))
                            }
                            _ => return Ok(Some(sequence[0] as i32)),
                        }
                    }
                }
                match sequence[2] {
                    b'A' => return Ok(Some(EditorKey::ArrowUp as i32)),
                    b'B' => return Ok(Some(EditorKey::ArrowDown as i32)),
                    b'C' => return Ok(Some(EditorKey::ArrowRight as i32)),
                    b'D' => return Ok(Some(EditorKey::ArrowLeft as i32)),
                    _ => return Ok(Some(sequence[0] as i32)),
                }
            }
        }
        Ok(Some(sequence[0] as i32))
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        self.append_buffer.append(HIDE_CURSOR);
        self.append_buffer.append(CURSOR_POSITION_START);
        self.draw_rows()?;
        Terminal::set_cursor_position_buffer(
            self.cursor_row + 1,
            self.cursor_col + 1,
            &mut self.append_buffer.chars,
        )?;
        self.append_buffer.append(SHOW_CURSOR);
        let buffer = take(&mut self.append_buffer.chars);
        self.terminal.write_output_from_buffer(buffer)?;
        Ok(())
    }

    fn move_cursor(&mut self, key: EditorKey) -> Result<(), Error> {
        match key {
            EditorKey::ArrowLeft => {
                Ok(self.cursor_col = self.cursor_col.saturating_sub(1))
            }
            EditorKey::ArrowRight => {
                if self.cursor_col < self.screen_cols - 1 {
                    return Ok(
                        self.cursor_col = self.cursor_col.saturating_add(1)
                    );
                }
                Ok(())
            }
            EditorKey::ArrowDown => {
                if self.cursor_row < self.screen_rows - 1 {
                    return Ok(
                        self.cursor_row = self.cursor_row.saturating_add(1)
                    );
                }
                Ok(())
            }
            EditorKey::ArrowUp => {
                Ok(self.cursor_row = self.cursor_row.saturating_sub(1))
            }
            EditorKey::PageUp => {
                while self.cursor_row > 0 {
                    self.cursor_row = self.cursor_row.saturating_sub(1);
                }
                return Ok(());
            }
            EditorKey::PageDown => {
                while self.cursor_row < self.screen_rows - 1 {
                    self.cursor_row = self.cursor_row.saturating_add(1);
                }
                Ok(())
            }
        }
    }

    pub fn new() -> Result<Self, Error> {
        let mut terminal = Terminal::new()?;
        terminal.enable_raw_mode()?;
        let (rows, cols) = terminal.get_window_size()?;
        let editor = Self {
            append_buffer: AppendBuffer::new(),
            cursor_col: 0,
            cursor_row: 0,
            screen_cols: cols,
            screen_rows: rows,
            terminal,
        };
        Ok(editor)
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            self.process_keypress()?;
        }
    }
}
