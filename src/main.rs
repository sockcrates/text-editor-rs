use termios::{Termios, tcgetattr, tcsetattr};

fn enable_raw_mode() {
    let mut termios = Termios::from_fd(0).unwrap();
    tcgetattr(0, &mut termios);
    termios.c_lflag &= !(termios::ECHO);
    tcsetattr(0, termios::TCSAFLUSH, &termios);
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
