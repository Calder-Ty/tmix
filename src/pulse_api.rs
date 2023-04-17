//! Code to communicate with Pulse Server
use pulse::{
    callbacks::ListResult,
    context::{
        introspect::{SinkInfo, SinkInputInfo},
        Context, FlagSet as ContextFlagSet,
    },
    def::Retval,
    mainloop::standard::{IterateResult, Mainloop},
    operation::Operation,
    proplist::Proplist,
};

use std::{
    cell::{RefCell, RefMut},
    collections::{hash_map, HashMap},
    io::Result as IOResult,
    rc::Rc,
};

use crate::data::{SinkInformation, SinkInputInformation};

/// Connects Sinks and their Input information
pub struct SinkAndInputs {
    sink: SinkInformation,
    sink_inputs: Vec<SinkInputInformation>,
}

impl SinkAndInputs {
    pub fn new(sink: SinkInformation, sink_inputs: Vec<SinkInputInformation>) -> Self {
        Self { sink, sink_inputs }
    }

    pub fn sink(&self) -> &SinkInformation {
        &self.sink
    }

    pub fn push(&mut self, value: SinkInputInformation) {
        self.sink_inputs.push(value)
    }

    pub fn iter(&self) -> std::slice::Iter<SinkInputInformation>{
        self.sink_inputs.iter()

    }
}

pub struct VolumeInfo {
    sinks_and_inputs: HashMap<u32, SinkAndInputs>,
}


impl VolumeInfo {
    fn new(sinks: Vec<SinkInformation>, input_info: Vec<SinkInputInformation>) -> Self {
        let mut sinks_and_inputs: HashMap<u32, SinkAndInputs> = HashMap::new();

        for sink in sinks {
            sinks_and_inputs.insert(sink.index, SinkAndInputs::new(sink, vec![]));
        }

        for input in input_info.into_iter() {
            if let Some(s) = sinks_and_inputs.get_mut(&input.sink) {
                s.push(input);
            }
            // If the sink doesn't exist, that seems like an issue in Pulse Audio, We aren't going
            // to display it
        }
        Self { sinks_and_inputs }
    }

    pub fn iter(&self) -> hash_map::Iter<u32, SinkAndInputs> {
        // FIXME: Unorderd!
        self.sinks_and_inputs.iter()
    }
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
        let (inputs_op, sink_inputs) = self.get_sink_inputs()?;
        let (info_op, sink_info) = self.get_sink_info()?;

        self.await_ops((inputs_op, info_op));

        // SAFTEY: It is ok to take because by this point the callbacks have
        // completed and we are ready to move on
        Ok(VolumeInfo::new(sink_info.take(), sink_inputs.take()))
    }

    /// Await for array of Ops to complete
    /// This allows us to make sure that all operations and callbacks of passed in ops have
    /// completed so that we can then safely move on
    // FIXME: This is a dumb way to do the generics, but for now it works. If i ever
    // Want to Add more kinds of ops i should take the time to figure it out.
    fn await_ops<T: ?Sized, U: ?Sized>(&mut self, ops: (Operation<T>, Operation<U>)) {
        loop {
            self.mainloop.iterate(false);
            match ops.0.get_state() {
                pulse::operation::State::Done | pulse::operation::State::Cancelled => break,
                pulse::operation::State::Running => {}
            }
            match ops.1.get_state() {
                pulse::operation::State::Done | pulse::operation::State::Cancelled => break,
                pulse::operation::State::Running => {}
            }
        }
    }

    fn get_sink_info(
        &mut self,
    ) -> IOResult<(
        Operation<dyn FnMut(ListResult<&SinkInfo>)>,
        Rc<RefCell<Vec<SinkInformation>>>,
    )> {
        let introspector = self.ctx.introspect();
        let results: Rc<RefCell<Vec<SinkInformation>>> = Rc::new(RefCell::new(vec![]));
        let results_inner = results.clone();
        let op = introspector.get_sink_info_list(move |res: ListResult<&SinkInfo>| match res {
            pulse::callbacks::ListResult::Item(source) => {
                let mut r: RefMut<Vec<SinkInformation>> = results_inner.borrow_mut();
                r.push(source.into());
            }
            pulse::callbacks::ListResult::End => {}
            pulse::callbacks::ListResult::Error => {
                eprintln!("ERROR: Mr. Robinson");
            }
        });

        Ok((op, results))
    }

    fn get_sink_inputs(
        &mut self,
    ) -> IOResult<(
        Operation<dyn FnMut(ListResult<&SinkInputInfo>)>,
        Rc<RefCell<Vec<SinkInputInformation>>>,
    )> {
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

        Ok((op, results))
    }

    pub fn shutdown(&mut self) {
        self.ctx.disconnect();
        // Clean shutdown
        self.mainloop.quit(Retval(0)); // uncertain whether this is necessary
    }
}
