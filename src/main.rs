use libc::{STDIN_FILENO, BRKINT};
use std::io::{stdin, stdout, Error, Read, Write};
use std::process::exit;
use termios::{
    tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP,
    IXON, OPOST, TCSAFLUSH,
};

fn exit_with_error(err: Error) {
    println!("Error: {}", err);
    exit(1);
}

fn disable_raw_mode(original_termios: &mut Termios) {
    match tcsetattr(STDIN_FILENO, TCSAFLUSH, original_termios) {
        Ok(_) => {}
        Err(e) => exit_with_error(e),
    }
}

fn enable_raw_mode(original_termios: &mut Termios) {
    match tcgetattr(STDIN_FILENO, original_termios) {
        Ok(_) => {}
        Err(e) => {
            exit_with_error(e);
        }
    }
    original_termios.c_cflag |= CS8;
    original_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    original_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    original_termios.c_oflag &= !(OPOST);
    match tcsetattr(STDIN_FILENO, TCSAFLUSH, original_termios) {
        Ok(_) => {}
        Err(e) => exit_with_error(e),
    }
}

fn main() {
    let mut original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(1);
    });
    enable_raw_mode(&mut original_termios);
    loop {
        let mut stdin = stdin();
        let stdout = stdout();
        let mut input: [u8; 1] = [0; 1];
        match stdout.lock().flush() {
            Ok(_) => {}
            Err(e) => exit_with_error(e),
        }
        match stdin.read_exact(&mut input) {
            Ok(_) => {}
            Err(e) => exit_with_error(e),
        }
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
