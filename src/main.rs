mod editor;
use std::process::exit;

use editor::Editor;

fn main() {
    let mut editor = Editor::new().unwrap_or_else(|e| {
        eprintln!("Error in main: {}", e);
        exit(1);
    });
    editor.run().unwrap_or_else(|e| {
        eprintln!("Error running terminal {}", e);
        exit(1);
    });
}
