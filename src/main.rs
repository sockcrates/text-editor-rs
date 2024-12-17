mod editor;
use std::io::Error;

use editor::Editor;

fn main() -> Result<(), Error> {
    let mut editor = Editor::try_new()?;
    editor.run()
}
