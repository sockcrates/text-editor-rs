use libc::STDIN_FILENO;
use std::error::Error;
use std::io::{stdin, stdout, Read, Stdin, Write};
use std::process::exit;
use termios::Termios;

mod terminal;

struct EditorConfig {
    raw_termios: Termios,
}

fn exit_with_error(location: &str, err: &dyn Error) {
    println!("Error in {}: {}", location, err);
    exit(1);
}

fn read_key(stdin: &mut Stdin) -> u8 {
    let mut input: [u8; 1] = [0; 1];
    match stdin.read(&mut input) {
        Ok(_) => {}
        Err(e) => {
            exit_with_error("reading stdin", &e);
        }
    };
    input[0]
}

fn process_keypress(key: u8, original_termios: &mut Termios) {
    match key {
        b'\x11' => {
            refresh_screen();
            terminal::disable_raw_mode(original_termios);
            exit(0);
        }
        b'\r' => print!("\r\n"),
        _ => print!("{}", key as char),
    }
}

fn draw_rows() {
    for _ in 0..24 {
        print!("~\r\n");
    }
}

fn refresh_screen() {
    print!("\x1b[2J");
    print!("\x1b[H");
    draw_rows();
    print!("\x1b[H");
}

fn main() {
    let mut original_termios = Termios::from_fd(STDIN_FILENO).unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(1);
    });
    let mut editor = EditorConfig {
        raw_termios: original_termios.clone(),
    };
    terminal::enable_raw_mode(&mut editor.raw_termios);
    refresh_screen();
    loop {
        let mut stdin = stdin();
        let stdout = stdout();
        stdout.lock().flush().unwrap_or_else(|e| {
            exit_with_error("flushing stdout", &e);
        });
        let key = read_key(&mut stdin);
        process_keypress(key, &mut original_termios);
    }
}
