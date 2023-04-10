//! Code to communicate with Pulse Server
use log::{debug, error};
use pulse::{
    callbacks::ListResult,
    channelmap,
    context::{
        introspect::{SinkInfo, SourceInfo},
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

type SourceData = Vec<SourceInformation>;
type SinkData = Vec<SinkInformation>;

#[derive(Clone, Debug)]
pub struct SinkInformation {
    /// Name of the sink.
    pub name: Option<String>,
    /// Index of the sink.
    pub index: u32,
    /// Description of this sink.
    pub description: Option<String>,
    /// Sample spec of this sink.
    pub sample_spec: sample::Spec,
    /// Channel map.
    pub channel_map: channelmap::Map,
    /// Index of the owning module of this sink, or `None` if is invalid.
    pub owner_module: Option<u32>,
    /// Volume of the sink.
    pub volume: ChannelVolumes,
    /// Mute switch of the sink.
    pub mute: bool,
    /// Index of the monitor source connected to this sink.
    pub monitor_source: u32,
    /// The name of the monitor source.
    pub monitor_source_name: Option<String>,
    /// Driver name.
    pub driver: Option<String>,
    /// Flags.
    pub flags: def::SinkFlagSet,
    /// Property list.
    pub proplist: Proplist,
    /// Some kind of “base” volume that refers to unamplified/unattenuated volume in the context of
    /// the output device.
    pub base_volume: Volume,
    /// State.
    pub state: def::SinkState,
    /// Number of volume steps for sinks which do not support arbitrary volumes.
    pub n_volume_steps: u32,
    /// Card index, or `None` if invalid.
    pub card: Option<u32>,
    /// Set of formats supported by the sink.
    pub formats: Vec<format::Info>,
}

impl From<&SinkInfo<'_>> for SinkInformation {
    fn from(value: &SinkInfo<'_>) -> Self {
        let name = value.name.as_ref().map(|x| x.to_string());
        let description = value.description.as_ref().map(|x| x.to_string());
        let monitor_source_name = value.monitor_source_name.as_ref().map(|x| x.to_string());
        let driver = value.driver.as_ref().map(|x| x.to_string());

        Self {
            name,
            index: value.index.clone(),
            description,
            sample_spec: value.sample_spec.clone(),
            channel_map: value.channel_map.clone(),
            owner_module: value.owner_module.clone(),
            volume: value.volume.clone(),
            mute: value.mute.clone(),
            monitor_source: value.monitor_source.clone(),
            monitor_source_name,
            driver,
            flags: value.flags.clone(),
            proplist: value.proplist.clone(),
            base_volume: value.base_volume.clone(),
            state: value.state.clone(),
            n_volume_steps: value.n_volume_steps.clone(),
            card: value.card.clone(),
            formats: value.formats.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceInformation {
    /// Name of the source.
    pub name: Option<String>,
    /// Index of the source.
    pub index: u32,
    /// Description of this source.
    pub description: Option<String>,
    /// Sample spec of this source.
    pub sample_spec: sample::Spec,
    /// Channel map.
    pub channel_map: channelmap::Map,
    /// Owning module index, or `None`.
    pub owner_module: Option<u32>,
    /// Volume of the source.
    pub volume: ChannelVolumes,
    /// Mute switch of the sink.
    pub mute: bool,
    /// If this is a monitor source, the index of the owning sink, otherwise `None`.
    pub monitor_of_sink: Option<u32>,
    /// Name of the owning sink, or `None`.
    pub monitor_of_sink_name: Option<String>,
    /// Length of filled record buffer of this source.
    pub latency: MicroSeconds,
    /// Driver name.
    pub driver: Option<String>,
    /// Flags.
    pub flags: def::SourceFlagSet,
    /// Property list.
    pub proplist: Proplist,
    /// The latency this device has been configured to.
    pub configured_latency: MicroSeconds,
    /// Some kind of “base” volume that refers to unamplified/unattenuated volume in the context of
    /// the input device.
    pub base_volume: Volume,
    /// State.
    pub state: def::SourceState,
    /// Number of volume steps for sources which do not support arbitrary volumes.
    pub n_volume_steps: u32,
    /// Card index, or `None`.
    pub card: Option<u32>,
    /// Set of formats supported by the sink.
    pub formats: Vec<format::Info>,
}

impl From<&SourceInfo<'_>> for SourceInformation {
    fn from(value: &SourceInfo<'_>) -> Self {
        let name = value.name.as_ref().map(|x| x.to_string());
        let description = value.description.as_ref().map(|x| x.to_string());
        let monitor_of_sink_name = value.monitor_of_sink_name.as_ref().map(|x| x.to_string());
        let driver = value.driver.as_ref().map(|x| x.to_string());

        Self {
            name,
            index: value.index.clone(),
            description,
            sample_spec: value.sample_spec.clone(),
            channel_map: value.channel_map.clone(),
            owner_module: value.owner_module.clone(),
            volume: value.volume.clone(),
            mute: value.mute.clone(),
            monitor_of_sink: value.monitor_of_sink.clone(),
            monitor_of_sink_name,
            driver,
            flags: value.flags.clone(),
            proplist: value.proplist.clone(),
            base_volume: value.base_volume.clone(),
            state: value.state.clone(),
            n_volume_steps: value.n_volume_steps.clone(),
            card: value.card.clone(),
            formats: value.formats.clone(),
            latency: value.latency.clone(),
            configured_latency: value.configured_latency.clone(),
        }
    }
}
/// Higher Level Pulse API
pub struct PulseAPI {
    mainloop: Mainloop,
    ctx: Context,
    tx: Sender<SourceInformation>,
}

impl PulseAPI {
    pub fn new(tx: Sender<SourceInformation>) -> Self {
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

    pub fn get_source_info<'a>(&mut self) {
        let introspector = self.ctx.introspect();
        let tx_inner = self.tx.clone();
        let op = introspector.get_source_info_list(move |res: ListResult<&SourceInfo>| match res {
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
