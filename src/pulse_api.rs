//! Code to communicate with Pulse Server
use log::{debug, error};
use pulse::{
    callbacks::ListResult,
    channelmap,
    context::{
        introspect::{SinkInfo, SinkInputInfo, SourceInfo},
        Context, FlagSet as ContextFlagSet,
    },
    def::{self, Retval},
    format,
    mainloop::standard::{IterateResult, Mainloop},
    proplist::Proplist,
    sample,
    time::MicroSeconds,
    volume::{ChannelVolumes, Volume},
};
use std::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
    io::Result as IOResult,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::mpsc::{Receiver, SendError, Sender},
};

type SinkData = Vec<SinkInputInformation>;


#[derive(Debug)]
pub struct SinkInputInformation {
    /// Index of the sink input.
    pub index: u32,
    /// Name of the sink input.
    pub name: Option<String>,
    /// Index of the module this sink input belongs to, or `None` when it does not belong to any
    /// module.
    pub owner_module: Option<u32>,
    /// Index of the client this sink input belongs to, or invalid when it does not belong to any
    /// client.
    pub client: Option<u32>,
    /// Index of the connected sink.
    pub sink: u32,
    /// The sample specification of the sink input.
    pub sample_spec: sample::Spec,
    /// Channel map.
    pub channel_map: channelmap::Map,
    /// The volume of this sink input.
    pub volume: ChannelVolumes,
    /// Latency due to buffering in sink input, see [`TimingInfo`](crate::def::TimingInfo) for
    /// details.
    pub buffer_usec: MicroSeconds,
    /// Latency of the sink device, see [`TimingInfo`](crate::def::TimingInfo) for details.
    pub sink_usec: MicroSeconds,
    /// The resampling method used by this sink input.
    pub resample_method: Option<String>,
    /// Driver name.
    pub driver: Option<String>,
    /// Stream muted.
    pub mute: bool,
    /// Property list.
    pub proplist: Proplist,
    /// Stream corked.
    pub corked: bool,
    /// Stream has volume. If not set, then the meaning of this struct’s volume member is
    /// unspecified.
    pub has_volume: bool,
    /// The volume can be set. If not set, the volume can still change even though clients can’t
    /// control the volume.
    pub volume_writable: bool,
    /// Stream format information.
    pub format: format::Info,
}

impl From<&SinkInputInfo<'_>> for SinkInputInformation {
    fn from(value: &SinkInputInfo<'_>) -> Self {
        let name = value.name.as_ref().map(|x| x.to_string());
        let resample_method = value.resample_method.as_ref().map(|x| x.to_string());
        let driver = value.driver.as_ref().map(|x| x.to_string());
        Self {
            index: value.index.clone(),
            name,
            owner_module: value.owner_module.clone(),
            client: value.client.clone(),
            sink: value.sink.clone(),
            sample_spec: value.sample_spec.clone(),
            channel_map: value.channel_map.clone(),
            volume: value.volume.clone(),
            buffer_usec: value.buffer_usec.clone(),
            sink_usec: value.sink_usec.clone(),
            resample_method,
            driver,
            mute: value.mute.clone(),
            proplist: value.proplist.clone(),
            corked: value.corked.clone(),
            has_volume: value.has_volume.clone(),
            volume_writable: value.volume_writable.clone(),
            format: value.format.clone(),
        }
    }
}

/// Higher Level Pulse API
pub struct PulseAPI {
    mainloop: Mainloop,
    ctx: Context,
    tx: Sender<SinkInputInformation>,
}

impl PulseAPI {
    pub fn new(tx: Sender<SinkInputInformation>) -> Self {
        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(pulse::proplist::properties::APPLICATION_NAME, "tmix")
            .unwrap();
        let mainloop = Mainloop::new().expect("Failed to create mainloop");

        let ctx = Context::new_with_proplist(&mainloop, "tmixContext", &proplist)
            .expect("Failed to create new context");

        PulseAPI { mainloop, ctx, tx }
    }

    pub fn startup_connection(&mut self) -> IOResult<()> {
        self.ctx
            .borrow_mut()
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        // Wait for context to be ready
        loop {
            match self.mainloop.borrow_mut().iterate(false) {
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

    pub fn get_sink_inputs<'a>(&mut self) {
        let introspector = self.ctx.introspect();
        let tx_inner = self.tx.clone();
        let op =
            introspector.get_sink_input_info_list(move |res: ListResult<&SinkInputInfo>| match res {
                pulse::callbacks::ListResult::Item(source) => match tx_inner.send(source.into()) {
                    Ok(_) => {}
                    Err(SendError(si)) => {
                        error!("SendError Was recieved, Reciever must be shut down. Shutting Down");
                        debug!("{:?}", si);
                    }
                },
                pulse::callbacks::ListResult::End => {}
                pulse::callbacks::ListResult::Error => {
                    eprintln!("ERROR: Mr. Robinson");
                }
            });
        loop {
            self.mainloop.borrow_mut().iterate(false);
            match op.get_state() {
                pulse::operation::State::Done | pulse::operation::State::Cancelled => break,
                pulse::operation::State::Running => {}
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.ctx.disconnect();
        // Clean shutdown
        self.mainloop.borrow_mut().quit(Retval(0)); // uncertain whether this is necessary
    }
}
