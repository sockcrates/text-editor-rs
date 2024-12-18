mod editor;
use std::io::Error;

use editor::Editor;

fn main() -> Result<(), Error> {
    let args = std::env::args().collect::<Vec<String>>();
    let file_name = if args.len() >= 2 {
        Some(args[1].as_str())
    } else {
        None
    };
    Editor::try_new()?.run(file_name)
}
