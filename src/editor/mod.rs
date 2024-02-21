use std::io::{stdin, stdout, Error, Read, Write};
use std::process::exit;
use std::u16;

mod append_buffer;
use append_buffer::AppendBuffer;

mod terminal;
use terminal::Terminal;

pub struct Editor {
    screen_cols: u16,
    screen_rows: u16,
    terminal: Terminal,
}

impl Editor {
    fn draw_rows(&self, append_buffer: &mut AppendBuffer) -> Result<(), Error> {
        for i in 0..self.screen_rows {
            append_buffer.append("~");
            if i < self.screen_rows - 1 {
                append_buffer.append("\r\n");
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.refresh_screen().unwrap_or_else(|e| {
            println!("Error refreshing screen: {}", e);
            exit(1);
        });
        self.terminal.disable_raw_mode().unwrap_or_else(|e| {
            println!("Error disabling raw mode: {}", e);
            exit(1);
        });
        exit(0);
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let key: u8 = Self::read_key()?;
        match key {
            b'\x11' => Ok(self.exit()),
            _ => Ok(()),
        }
    }

    fn read_key() -> Result<u8, Error> {
        let mut stdin = stdin();
        let mut input: [u8; 1] = [0; 1];
        stdin.read(&mut input)?;
        Ok(input[0])
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        let mut append_buffer = AppendBuffer::new(); 
        append_buffer.append("\x1b[2J");
        append_buffer.append("\x1b[H");
        self.draw_rows(&mut append_buffer)?;
        append_buffer.append("\x1b[H");
        let mut stdout = stdout();
        stdout.write(&append_buffer.buffer)?;
        append_buffer.free();
        Ok(())
    }

    pub fn new() -> Result<Self, Error> {
        let mut terminal = Terminal::new()?;
        terminal.enable_raw_mode()?;
        let (rows, cols) = Terminal::get_window_size()?;
        let editor = Self {
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
