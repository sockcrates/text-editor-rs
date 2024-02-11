use libc::{ioctl, winsize, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ};
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
    fn draw_rows(&mut self) {
        for _ in 0..self.screen_rows {
            print!("~\r\n");
        }
    }

    fn exit_with_error(location: &str, err: &dyn Error) {
        println!("Error in {}: {}", location, err);
        exit(1);
    }

    fn get_cursor_position() {
        write!(stdout(), "\x1b[6n").unwrap_or_else(|e| {
            Self::exit_with_error("writing ANSI escape Device Status Report, 6", &e);
        });
        stdout().flush().unwrap_or_else(|e| {
            Self::exit_with_error("flushing stdout", &e);
        });
        let mut buf: [u8; 1] = [0; 1];
        loop {
            match stdin().read(&mut buf) {
                Ok(bytes) => {
                    if bytes != 1 {
                        break;
                    }
                }
                Err(e) => Self::exit_with_error("reading stdin", &e),
            }
        }
    }

    fn get_window_size(&mut self) {
        let mut ws: winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) == -1 || ws.ws_col == 0 {
                write!(stdout(), "\x1b[999C\x1b[999B").unwrap_or_else(|e| {
                    Self::exit_with_error("writing ANSI escape sequence - place cursor at end", &e);
                });
                Self::get_cursor_position();
            } else {
                self.screen_cols = ws.ws_col;
                self.screen_rows = ws.ws_row;
            }
        }
    }

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
        let mut editor = Self {
            original_termios,
            raw_termios: original_termios.clone(),
            screen_cols: 0,
            screen_rows: 0,
        };
        terminal::enable_raw_mode(&mut editor.raw_termios)?;
        editor.get_window_size();
        editor.refresh_screen();
        Ok(editor)
    }

    pub fn run(&mut self) {
        loop {
            let mut stdin = stdin();
            let stdout = stdout();
            stdout.lock().flush().unwrap_or_else(|e| {
                Self::exit_with_error("flushing stdout", &e);
            });
            let key = Self::read_key(&mut stdin).unwrap_or_else(|e| {
                Self::exit_with_error("reading key", &e);
                0
            });
            self.process_keypress(key);
        }
    }
}
