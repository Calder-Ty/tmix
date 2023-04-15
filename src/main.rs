mod app;

use std::{cell::RefCell, io::Result, net::Shutdown, ops::Deref, rc::Rc, sync::mpsc, thread, time::Duration};

use app::{App, UIMessages};
use log::error;
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

    // Setup Connection to Pulse
    let mut applicaton = App::try_new()?;
    applicaton.run()?;
    Ok(())
}
