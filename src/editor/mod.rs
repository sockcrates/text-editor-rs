use std::io::{Error, ErrorKind};
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
        let key: u8 = self.read_key()?;
        match key {
            b'\x11' => Ok(self.exit()),
            b'a' | b'd' | b's' | b'w' => self.move_cursor(key),
            _ => Ok(()),
        }
    }

    fn read_key(&mut self) -> Result<u8, Error> {
        self.terminal.read_input()
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

    fn move_cursor(&mut self, key: u8) -> Result<(), Error> {
        match key {
            b'a' => Ok(self.cursor_col = self.cursor_col.saturating_sub(1)),
            b'd' => Ok(self.cursor_col = self.cursor_col.saturating_add(1)),
            b's' => Ok(self.cursor_row = self.cursor_row.saturating_add(1)),
            b'w' => Ok(self.cursor_row = self.cursor_row.saturating_sub(1)),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid key")),
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
