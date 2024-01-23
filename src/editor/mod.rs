use libc::{ioctl, winsize, STDIN_FILENO, TIOCGWINSZ};
use std::error::Error;
use std::io::{stdin, stdout, Read, Stdin, Write};
use std::process::exit;
use termios::Termios;

mod terminal;

pub struct Editor {
    original_termios: Termios,
    raw_termios: Termios,
    screen_rows: u16,
    screen_cols: u16,
}

impl Editor {
    fn draw_rows(&mut self) {
        for _ in 0..self.screen_rows {
            print!("~\r\n");
        }
    }

    fn exit_with_error(location: &str, err: &dyn Error) {
        println!("Error in {}: {}", location, err);
        exit(1);
    }

    fn get_window_size(&mut self) {
        let mut ws: winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            ioctl(STDIN_FILENO, TIOCGWINSZ, &mut ws);
        }
        self.screen_rows = ws.ws_row;
        self.screen_cols = ws.ws_col;
    }

    fn graceful_exit(&mut self) {
        self.refresh_screen();
        terminal::disable_raw_mode(&mut self.original_termios);
        exit(0);
    }

    fn process_keypress(&mut self, key: u8) {
        match key {
            b'\x11' => self.graceful_exit(),
            b'\r' => print!("\r\n"),
            _ => print!("{}", key as char),
        }
    }

    fn read_key(stdin: &mut Stdin) -> u8 {
        let mut input: [u8; 1] = [0; 1];
        match stdin.read(&mut input) {
            Ok(_) => {}
            Err(e) => {
                Self::exit_with_error("reading stdin", &e);
            }
        };
        input[0]
    }

    fn refresh_screen(&mut self) {
        print!("\x1b[2J");
        print!("\x1b[H");
        self.draw_rows();
        print!("\x1b[H");
    }

    pub fn new() -> Self {
        let original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
            println!("Error: {}", e);
            exit(1);
        });
        let mut editor = Self {
            original_termios,
            raw_termios: original_termios.clone(),
            screen_rows: 0,
            screen_cols: 0,
        };
        terminal::enable_raw_mode(&mut editor.raw_termios);
        editor.get_window_size();
        editor.refresh_screen();
        editor
    }

    pub fn run(&mut self) {
        loop {
            let mut stdin = stdin();
            let stdout = stdout();
            stdout.lock().flush().unwrap_or_else(|e| {
                Self::exit_with_error("flushing stdout", &e);
            });
            let key = Self::read_key(&mut stdin);
            self.process_keypress(key);
        }
    }
}
