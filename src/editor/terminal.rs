use libc::STDIN_FILENO;
use std::io::Error;
use termios::{
    tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP,
    IXON, OPOST, TCSAFLUSH, VMIN, VTIME,
};

pub fn disable_raw_mode(original_termios: &mut Termios) -> Result<(), Error> {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, original_termios)?;
    Ok(())
}

pub fn enable_raw_mode(raw_mode_termios: &mut Termios) -> Result<(), Error> {
    tcgetattr(STDIN_FILENO, raw_mode_termios)?;
    raw_mode_termios.c_cflag |= CS8;
    raw_mode_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw_mode_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw_mode_termios.c_oflag &= !(OPOST);
    raw_mode_termios.c_cc[VMIN] = 0;
    raw_mode_termios.c_cc[VTIME] = 1;
    tcsetattr(STDIN_FILENO, TCSAFLUSH, raw_mode_termios)?;
    Ok(())
}
