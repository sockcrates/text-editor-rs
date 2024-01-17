use libc::STDIN_FILENO;
use termios::{tcgetattr, tcsetattr, Termios};

fn enable_raw_mode() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
        println!("Error: {}", e);
        std::process::exit(1);
    });
    match tcgetattr(STDIN_FILENO, &mut termios) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
    termios.c_lflag &= !(termios::ECHO);
    match tcsetattr(STDIN_FILENO, termios::TCSAFLUSH, &termios) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    enable_raw_mode();
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(e) => println!("Error: {}", e),
        }

        match input.trim() {
            "q" => std::process::exit(0),
            _ => println!("{}", input.trim()),
        }
    }
}
