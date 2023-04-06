mod app;

use std::{cell::RefCell, io::Result, ops::Deref, rc::Rc};

use app::App;
use pulse::{
    context::{Context, FlagSet as ContextFlagSet},
    def::Retval,
    mainloop::standard::{IterateResult, Mainloop},
    proplist::Proplist,
    sample::{Format, Spec},
    volume::ChannelVolumes,
};

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    // Setup the Terminal (Per Docs)
    let mut applicaton = App::default();
    // applicaton.run()?;
    // Testing the Pulse Audio Volume
    // let mut channels = ChannelVolumes::default();
    // let inited = channels.init();
    // println!("Valid: {:?}", inited.is_valid());
    // println!("Len: {:?}", inited.len());
    // println!("Channesl: {:?}", inited.get());
    // println!("Self: {:?}", inited);
    // println!("{:?}", ChannelVolumes::CHANNELS_MAX);

    let spec = Spec {
        format: Format::S16NE,
        channels: 2,
        rate: 44100,
    };
    assert!(spec.is_valid());

    let mut proplist = Proplist::new().unwrap();
    proplist
        .set_str(pulse::proplist::properties::APPLICATION_NAME, "FooApp")
        .unwrap();

    let mut mainloop = Rc::new(RefCell::new(
        Mainloop::new().expect("Failed to create mainloop"),
    ));

    let mut context = Rc::new(RefCell::new(
        Context::new_with_proplist(mainloop.borrow().deref(), "FooAppContext", &proplist)
            .expect("Failed to create new context"),
    ));

    context
        .borrow_mut()
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .expect("Failed to connect context");

    // Wait for context to be ready
    loop {
        match mainloop.borrow_mut().iterate(false) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                eprintln!("Iterate state was not success, quitting...");
                return Ok(());
            }
            IterateResult::Success(_) => {}
        }
        match context.borrow().get_state() {
            pulse::context::State::Ready => {
                break;
            }
            pulse::context::State::Failed | pulse::context::State::Terminated => {
                eprintln!("Context state failed/terminated, quitting...");
                return Ok(());
            }
            _ => {}
        }
    }

    let introspector = context.borrow().introspect();
    let op = introspector.get_sink_info_list(|res| {
        match res {
            pulse::callbacks::ListResult::Item(v) => {
                v.volume;
                println!("Valid: {:?}", v.volume.is_valid());
                println!("Len: {:?}", v.volume.len());
                println!("Channesl: {:?}", v.volume.get());
                println!("{:?}", v.volume.get()[0].print());
                println!("{:?}", ChannelVolumes::CHANNELS_MAX);
            }
            pulse::callbacks::ListResult::End => {},
            pulse::callbacks::ListResult::Error => {
                eprintln!("ERROR: Mr. Robinson");
            }
        }

    });
    loop {
        mainloop.borrow_mut().iterate(false);
        match op.get_state() {
            pulse::operation::State::Running => {},
            pulse::operation::State::Done | pulse::operation::State::Cancelled => break
        }
    }

    // Clean shutdown
    mainloop.borrow_mut().quit(Retval(0)); // uncertain whether this is necessary
    Ok(())
}
