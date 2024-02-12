use libc::{ioctl, winsize, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ};
use std::io::Error;
use termios::{
    tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP,
    IXON, OPOST, TCSAFLUSH, VMIN, VTIME,
};

pub struct Terminal {
    original_termios: Termios,
    raw_termios: Termios,
}

impl Terminal {
    pub fn clear_screen() {
        print!("\x1b[2J");
        print!("\x1b[H");
    }

    pub fn cursor_home() {
        print!("\x1b[H");
    }

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

    pub fn get_window_size() -> Result<(u16, u16), Error> {
        let mut ws: winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) == -1 || ws.ws_col == 0 {
                Ok((0, 0))
            } else {
                Ok((ws.ws_col, ws.ws_row))
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
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.disable_raw_mode().unwrap_or_else(|e| {
            println!("Error disabling raw mode: {}", e);
            std::process::exit(1);
        });
    }
}

