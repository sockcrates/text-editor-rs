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
    fn draw_rows(&self) {
        for i in 0..self.screen_rows {
            print!("~");
            if i < self.screen_rows - 1 {
                print!("\r\n");
            }
        }
    }

    fn exit(&mut self) {
        self.refresh_screen();
        self.terminal.disable_raw_mode().unwrap_or_else(|e| {
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

    fn read_key(stdin: &mut Stdin) -> Result<u8, Error> {
        let mut input: [u8; 1] = [0; 1];
        stdin.read(&mut input)?;
        Ok(input[0])
    }

    fn refresh_screen(&mut self) {
        Terminal::clear_screen();
        self.draw_rows();
        Terminal::cursor_home();
    }

    pub fn new() -> Result<Self, Error> {
        let mut terminal = Terminal::new()?;
        terminal.enable_raw_mode()?;
        let (rows, cols) = Terminal::get_window_size()?;
        let mut editor = Self {
            screen_cols: cols,
            screen_rows: rows,
            terminal,
        };
        editor.refresh_screen();
        Ok(editor)
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let mut stdin = stdin();
            let stdout = stdout();
            stdout.lock().flush()?;
            let key = Self::read_key(&mut stdin)?;
            self.process_keypress(key);
        }
    }
}
