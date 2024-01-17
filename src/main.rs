use libc::STDIN_FILENO;
use termios::{tcgetattr, tcsetattr, Termios};

fn exit_with_error(err: std::io::Error) {
    println!("Error: {}", err);
    std::process::exit(1);
}

fn enable_raw_mode(original_termios: &mut Termios) {
    match tcgetattr(STDIN_FILENO, original_termios) {
        Ok(_) => {}
        Err(e) => {
            exit_with_error(e);
        }
    }
    original_termios.c_lflag &= !(termios::ECHO);
    match tcsetattr(STDIN_FILENO, termios::TCSAFLUSH, original_termios) {
        Ok(_) => {}
        Err(e) => exit_with_error(e),
    }
}

fn main() {
    let mut original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
        println!("Error: {}", e);
        std::process::exit(1);
    });
    enable_raw_mode(&mut original_termios);
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(e) => {
                exit_with_error(e);
            }
        }

        match input.trim() {
            "q" => std::process::exit(0),
            _ => println!("{}", input.trim()),
        }
    }
}
