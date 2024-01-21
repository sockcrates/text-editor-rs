use libc::STDIN_FILENO;
use std::error::Error;
use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use termios::Termios;

mod terminal;

fn exit_with_error(location: &str, err: &dyn Error) {
    println!("Error in {}: {}", location, err);
    exit(1);
}

fn main() {
    let mut original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(1);
    });
    let mut clone_termios = original_termios.clone();
    terminal::enable_raw_mode(&mut clone_termios);
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
            b'\x11' => {
                terminal::disable_raw_mode(&mut original_termios);
                exit(0);
            }
            b'\r' => print!("\r\n"),
            _ => print!("{}", input[0] as char),
        }
    }
}
