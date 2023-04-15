//! Code to communicate with Pulse Server
use pulse::{
    callbacks::ListResult,
    context::{introspect::{SinkInputInfo, SinkInfo}, Context, FlagSet as ContextFlagSet},
    def::Retval,
    mainloop::standard::{IterateResult, Mainloop},
    proplist::Proplist,
};
use std::{
    cell::{RefCell, RefMut},
    io::Result as IOResult,
    rc::Rc,
};

use crate::data::{SinkInputInformation, SinkInformation};

pub struct VolumeInfo {

    pub sink_inputs: Vec<SinkInputInformation>,
    pub sink_info: Vec<SinkInformation>,
}

/// Higher Level Pulse API
pub struct PulseAPI {
    mainloop: Mainloop,
    ctx: Context,
}




impl PulseAPI {
    pub fn new() -> Self {
        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(pulse::proplist::properties::APPLICATION_NAME, "tmix")
            .unwrap();
        let mainloop = Mainloop::new().expect("Failed to create mainloop");

        let ctx = Context::new_with_proplist(&mainloop, "tmixContext", &proplist)
            .expect("Failed to create new context");

        PulseAPI { mainloop, ctx }
    }

    pub fn startup_connection(&mut self) -> IOResult<()> {
        self.ctx
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        // Wait for context to be ready
        loop {
            match self.mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    eprintln!("Iterate state was not success, quitting...");
                    return Ok(());
                }
                IterateResult::Success(_) => {}
            }
            match self.ctx.get_state() {
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
        Ok(())
    }

    pub fn get_volume_info(&mut self) -> IOResult<VolumeInfo> {

        let sink_info = self.get_sink_info()?;
        let sink_inpust = self.get_sink_inputs()?;

        // SAFTEY: It is ok to take because by this point the callbacks have 
        // completed and we are ready to move on
        Ok(VolumeInfo { sink_inputs: sink_inpust.take(), sink_info: sink_info.take()})

    }

    pub fn get_sink_info(&mut self) -> IOResult<Rc<RefCell<Vec<SinkInformation>>>> {

        let introspector = self.ctx.introspect();
        let results: Rc<RefCell<Vec<SinkInformation>>> = Rc::new(RefCell::new(vec![]));
        let results_inner = results.clone();
        let op =
            introspector.get_sink_info_list(
                move |res: ListResult<&SinkInfo>| match res {
                    pulse::callbacks::ListResult::Item(source) => {
                        let mut r: RefMut<Vec<SinkInformation>> = results_inner.borrow_mut();
                        r.push(source.into());
                    }
                    pulse::callbacks::ListResult::End => {}
                    pulse::callbacks::ListResult::Error => {
                        eprintln!("ERROR: Mr. Robinson");
                    }
                },
            );

        loop {
            self.mainloop.iterate(false);
            match op.get_state() {
                pulse::operation::State::Done | pulse::operation::State::Cancelled => break,
                pulse::operation::State::Running => {}
            }
        }

        Ok(results)
    }

    pub fn get_sink_inputs(&mut self) -> IOResult<Rc<RefCell<Vec<SinkInputInformation>>>> {
        let introspector = self.ctx.introspect();
        let results: Rc<RefCell<Vec<SinkInputInformation>>> = Rc::new(RefCell::new(vec![]));
        let results_inner = results.clone();
        let op =
            introspector.get_sink_input_info_list(
                move |res: ListResult<&SinkInputInfo>| match res {
                    pulse::callbacks::ListResult::Item(source) => {
                        let mut r: RefMut<Vec<SinkInputInformation>> = results_inner.borrow_mut();
                        r.push(source.into());
                    }
                    pulse::callbacks::ListResult::End => {}
                    pulse::callbacks::ListResult::Error => {
                        eprintln!("ERROR: Mr. Robinson");
                    }
                },
            );

        loop {
            self.mainloop.iterate(false);
            match op.get_state() {
                pulse::operation::State::Done | pulse::operation::State::Cancelled => break,
                pulse::operation::State::Running => {}
            }
        }

        Ok(results)
    }

    pub fn shutdown(&mut self) {
        self.ctx.disconnect();
        // Clean shutdown
        self.mainloop.quit(Retval(0)); // uncertain whether this is necessary
    }
}
