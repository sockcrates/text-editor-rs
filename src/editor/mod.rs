use std::io::{stdin, stdout, Error, Read, Stdin, Write};
use std::process::exit;
use std::u16;

mod terminal;
use terminal::Terminal;

pub struct Editor {
    screen_cols: u16,
    screen_rows: u16,
    terminal: Terminal,
}

impl Editor {
    fn draw_rows(&self) -> Result<(), Error> {
        let mut stdout = stdout();
        for i in 0..self.screen_rows {
            stdout.write(b"~")?;
            if i < self.screen_rows - 1 {
                stdout.write(b"\r\n")?;
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

    fn process_keypress(&mut self, key: u8) {
        match key {
            b'\x11' => self.exit(),
            _ => (),
        }
    }

    fn read_key() -> Result<u8, Error> {
        let mut stdin = stdin();
        let mut input: [u8; 1] = [0; 1];
        stdin.read(&mut input)?;
        Ok(input[0])
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::clear_screen()?;
        self.draw_rows();
        Terminal::cursor_home()?;
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
            let key = Self::read_key()?;
            self.process_keypress(key);
        }
    }
}
