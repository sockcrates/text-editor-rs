use libc::{ioctl, winsize, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ};
use std::io::{Error, Read, Write};
use termios::{
    tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP,
    IXON, OPOST, TCSAFLUSH, VMIN, VTIME,
};

/// VT100 escape sequence command "J" Erase in Display with the argument 2 (
/// clear whole screen)
pub const CLEAR_WHOLE_SCREEN: &str = "\x1b[2J";
/// VT100 escape sequence command "H" Cursor Position with the default argument
/// (1;1) (move cursor to the upper left corner)
pub const CURSOR_POSITION_START: &str = "\x1b[H";
/// VT100 escape sequence command "K" Erase in Line with the default argument 0
/// (clear whole line)
pub const ERASE_LINE: &str = "\x1b[K";
/// VT100 escape sequence command "l" Reset Mode
pub const HIDE_CURSOR: &str = "\x1b[?25l";
/// VT100 escape sequence command "h" Set Mode
pub const SHOW_CURSOR: &str = "\x1b[?25h";

pub struct Terminal {
    original_termios: Termios,
    raw_termios: Termios,
}

impl Terminal {
    pub fn disable_raw_mode(&mut self) -> Result<(), Error> {
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &mut self.original_termios)?;
        Ok(())
    }

    pub fn enable_raw_mode(&mut self) -> Result<(), Error> {
        tcgetattr(STDIN_FILENO, &mut self.raw_termios)?;
        self.raw_termios.c_cflag |= CS8;
        self.raw_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        self.raw_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        self.raw_termios.c_oflag &= !(OPOST);
        self.raw_termios.c_cc[VMIN] = 0;
        self.raw_termios.c_cc[VTIME] = 1;
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.raw_termios)?;
        Ok(())
    }

    pub fn get_cursor_position() -> Result<(u16, u16), Error> {
        let mut buf: [u8; 32] = [0; 32];
        let mut i = 0;
        let mut stdout = std::io::stdout();
        stdout.write(b"\x1b[6n")?;
        stdout.flush()?;
        while i < buf.len() - 1 {
            let mut byte: [u8; 1] = [0; 1];
            let nbytes = std::io::stdin().read(&mut byte)?;
            if nbytes == 0 || byte[0] == b'R' {
                break;
            }
            buf[i] = byte[0];
            i += 1;
        }
        let response = std::str::from_utf8(&buf[..i])
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        if !(response.starts_with("\x1b[") || response.ends_with('R')) {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid cursor response",
            ));
        }
        let mut parts_iter = response[2..].trim_end_matches('R').split(';');
        let row_str = parts_iter.next();
        let col_str = parts_iter.next();
        if let (Some(row_str), Some(col_str)) = (row_str, col_str) {
            let row = row_str.parse::<u16>().map_err(|e| {
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid row number: {}", e),
                )
            })?;
            let col = col_str.parse::<u16>().map_err(|e| {
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid column number: {}", e),
                )
            })?;
            return Ok((row, col));
        }
        Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid cursor response",
        ))
    }

    pub fn get_window_size() -> Result<(u16, u16), Error> {
        let mut ws: winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) == -1 || ws.ws_col == 0 {
                let mut stdout = std::io::stdout();
                stdout.write(b"\x1b[999C\x1b[999B")?;
                stdout.flush()?;
                Terminal::get_cursor_position()
            } else {
                Ok((ws.ws_row, ws.ws_col))
            }
        }
    }

    pub fn new() -> Result<Self, Error> {
        let original_termios = Termios::from_fd(STDIN_FILENO)?;
        let raw_termios = original_termios.clone();
        Ok(Self {
            original_termios,
            raw_termios,
        })
    }

    pub fn set_cursor_position_buffer(
        row: u16,
        col: u16,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        buffer.append(&mut format!("\x1b[{};{}H", row, col).into_bytes());
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.disable_raw_mode().unwrap_or_else(|e| {
            println!("Error disabling raw mode: {}", e);
            std::process::exit(1);
        });
    }
}
