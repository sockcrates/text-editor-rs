use libc::STDIN_FILENO;
use std::error::Error;
use std::process::exit;
use termios::{
    tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP,
    IXON, OPOST, TCSAFLUSH, VMIN, VTIME,
};

fn exit_with_error(location: &str, err: &dyn Error) {
    println!("Error in {}: {}", location, err);
    exit(1);
}

pub fn disable_raw_mode(original_termios: &mut Termios) {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, original_termios).unwrap_or_else(|e| {
        exit_with_error("disabling raw mode", &e);
    });
}

pub fn enable_raw_mode(raw_mode_termios: &mut Termios) {
    tcgetattr(STDIN_FILENO, raw_mode_termios).unwrap_or_else(|e| {
        exit_with_error("acquiring terminal", &e);
    });
    raw_mode_termios.c_cflag |= CS8;
    raw_mode_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw_mode_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw_mode_termios.c_oflag &= !(OPOST);
    raw_mode_termios.c_cc[VMIN] = 0;
    raw_mode_termios.c_cc[VTIME] = 1;
    tcsetattr(STDIN_FILENO, TCSAFLUSH, raw_mode_termios).unwrap_or_else(|e| {
        exit_with_error("enabling raw mode", &e);
    });
}
