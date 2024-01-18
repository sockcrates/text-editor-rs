use libc::STDIN_FILENO;
use std::io::{stdin, Error, Read};
use std::process::exit;
use termios::{tcgetattr, tcsetattr, Termios, ECHO, ICANON, TCSAFLUSH};

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
    original_termios.c_lflag &= !(ECHO | ICANON);
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
        let stdin = stdin();
        let mut input = String::new();
        for c in stdin.bytes() {
            match c {
                // Using ^F (ACK) to exit as ^Q (DC1) doesn't seem to work.
                Ok(b'\x06') => {
                    disable_raw_mode(&mut original_termios);
                    exit(0);
                }
                Ok(c) => {
                    input.push(c as char);
                    print!("{}", c as char);
                }
                Err(e) => exit_with_error(e),
            }
        }
    }
}
