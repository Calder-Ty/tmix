mod app;

use std::io::Result;

use app::App;

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // Setup Connection to Pulse
    let mut applicaton = App::try_new()?;
    applicaton.run()?;
    Ok(())
}
