use termios::{Termios, tcgetattr, tcsetattr};

fn enable_raw_mode() {
    let mut termios = Termios::from_fd(0).unwrap();
    match tcgetattr(0, &mut termios) {
        Ok(_) => {},
        Err(e) => println!("Error: {}", e),
    }
    termios.c_lflag &= !(termios::ECHO);
    match tcsetattr(0, termios::TCSAFLUSH, &termios) {
        Ok(_) => {},
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    enable_raw_mode();
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {},
            Err(e) => println!("Error: {}", e),
        }

        match input.trim() {
            "q" => std::process::exit(0),
            _ => println!("{}", input.trim()),
        }
    }
}
