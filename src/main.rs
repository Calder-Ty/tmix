mod app;

use std::io::Result;

use app::App;

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    // Setup the Terminal (Per Docs)
    let mut applicaton = App::default();
    applicaton.run()?;

    Ok(())
}
