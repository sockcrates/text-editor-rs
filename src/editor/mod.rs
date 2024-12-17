use std::io::Error;
use std::mem::take;
use std::process::exit;

mod append_buffer;
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
    num_rows: i32,
    screen_cols: i32,
    screen_rows: i32,
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
        let key = self.read_key()?;
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
            home if home == EditorKey::Home as i32 => {
                return self.move_cursor(EditorKey::Home);
            }
            end if end == EditorKey::End as i32 => {
                return self.move_cursor(EditorKey::End);
            }
            page_up if page_up == EditorKey::PageUp as i32 => {
                return self.move_cursor(EditorKey::PageUp);
            }
            page_down if page_down == EditorKey::PageDown as i32 => {
                return self.move_cursor(EditorKey::PageDown);
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
            EditorKey::ArrowUp => {
                Ok(self.cursor_row = self.cursor_row.saturating_sub(1))
            }
            EditorKey::ArrowDown => {
                if self.cursor_row < self.screen_rows - 1 {
                    return Ok(
                        self.cursor_row = self.cursor_row.saturating_add(1)
                    );
                }
                Ok(())
            }
            EditorKey::Home => Ok(self.cursor_col = 0),
            EditorKey::End => Ok(self.cursor_col = self.screen_cols - 1),
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
            _ => Ok(()),
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
            num_rows: 0,
            screen_cols: cols,
            screen_rows: rows,
            terminal,
        };
        Ok(editor)
    }

    fn open(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            self.process_keypress()?;
        }
    }
}
