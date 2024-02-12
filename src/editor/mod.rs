use libc::STDIN_FILENO;
use std::error::Error;
use std::io::{stdin, stdout, Error as IoError, Read, Stdin, Write};
use std::process::exit;
use std::u16;
use termios::Termios;

mod terminal;

pub struct Editor {
    original_termios: Termios,
    raw_termios: Termios,
    screen_cols: u16,
    screen_rows: u16,
}

impl Editor {
    fn draw_rows(&self) {
        for _ in 0..self.screen_rows {
            print!("~\r\n");
        }
    }

    fn get_cursor_position() {}

    fn exit(&mut self) {
        self.refresh_screen();
        terminal::disable_raw_mode(&mut self.original_termios).unwrap_or_else(|e| {
            println!("Error disabling raw mode: {}", e);
            exit(1);
        });
        exit(0);
    }

    fn process_keypress(&mut self, key: u8) {
        match key {
            b'\x11' => self.exit(),
            b'\r' => print!("\r\n"),
            _ => print!("{}", key as char),
        }
    }

    fn read_key(stdin: &mut Stdin) -> Result<u8, IoError> {
        let mut input: [u8; 1] = [0; 1];
        stdin.read(&mut input)?;
        Ok(input[0])
    }

    fn refresh_screen(&mut self) {
        print!("\x1b[2J");
        print!("\x1b[H");
        self.draw_rows();
        print!("\x1b[H");
    }

    pub fn new() -> Result<Self, IoError> {
        let original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
            println!("Error: {}", e);
            exit(1);
        });
        let mut clone_termios = original_termios.clone();
        terminal::enable_raw_mode(&mut clone_termios)?;
        let (cols, rows) = terminal::get_window_size()?;
        let mut editor = Self {
            original_termios,
            raw_termios: clone_termios,
            screen_cols: cols,
            screen_rows: rows,
        };
        editor.refresh_screen();
        Ok(editor)
    }

    pub fn run(&mut self) -> Result<(), IoError> {
        loop {
            let mut stdin = stdin();
            let stdout = stdout();
            stdout.lock().flush()?;
            let key = Self::read_key(&mut stdin)?;
            self.process_keypress(key);
        }
    }
}
