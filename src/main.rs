mod app;

use std::{cell::RefCell, io::Result, ops::Deref, rc::Rc, thread, sync::mpsc};

use app::App;
use pulse::{
    context::{Context, FlagSet as ContextFlagSet},
    def::Retval,
    mainloop::standard::{IterateResult, Mainloop},
    proplist::Proplist,
    sample::{Format, Spec},
    volume::ChannelVolumes,
};

use tmix::pulse_api::PulseAPI;

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    // Setup the Terminal (Per Docs)
    let mut applicaton = App::default();

    // Setup Connection to Pulse
    let (tx, rx) = mpsc::channel();
    let _pulse_thread = thread::spawn(|| {
        let mut api = PulseAPI::new(tx);
        api.startup_connection();
        let mut count = 0;
        while count < 1{
            api.get_source_info();
            count += 1;
        }
        api.shutdown()
    });

    loop {
        match rx.recv() {
            Ok(si) => {dbg!(si);},
            Err(_) => {
                eprintln!("Sender shutdown");
                break;
            }
        }
    }
    // applicaton.run()?;
    Ok(())
}
