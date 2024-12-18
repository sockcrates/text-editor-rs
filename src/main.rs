mod editor;
use std::io::Error;

use editor::Editor;

fn main() -> Result<(), Error> {
    Editor::try_new()?.run()
}
