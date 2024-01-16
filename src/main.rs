fn main() {
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
