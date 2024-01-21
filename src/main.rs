use libc::STDIN_FILENO;
use std::error::Error;
use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use termios::{
    tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP,
    IXON, OPOST, TCSAFLUSH, VMIN, VTIME,
};

fn exit_with_error(location: &str, err: &dyn Error) {
    println!("Error in {}: {}", location, err);
    exit(1);
}

fn disable_raw_mode(original_termios: &mut Termios) {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, original_termios).unwrap_or_else(|e| {
        exit_with_error("disabling raw mode", &e);
    });
}

fn enable_raw_mode(raw_mode_termios: &mut Termios) {
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

fn main() {
    let mut original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(1);
    });
    let mut clone_termios = original_termios.clone();
    enable_raw_mode(&mut clone_termios);
    loop {
        let mut stdin = stdin();
        let stdout = stdout();
        let mut input: [u8; 1] = [0; 1];
        stdout.lock().flush().unwrap_or_else(|e| {
            exit_with_error("flushing stdout", &e);
        });
        match stdin.read(&mut input) {
            Ok(_) => {}
            Err(e) => exit_with_error("reading from input", &e),
        };
        match input[0] {
            b'q' => {
                disable_raw_mode(&mut original_termios);
                exit(0);
            }
            b'\r' => print!("\r\n"),
            _ => print!("{}", input[0] as char),
        }
    }
}
