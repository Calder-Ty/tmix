mod app;

use std::io::Result;

use app::App;

fn main() -> Result<()> {
    // Setup the Terminal (Per Docs)
    let mut applicaton = App::default();
    applicaton.run()?;

    Ok(())
}
