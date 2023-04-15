//! Data Structures for Pulse Audio
use pulse::{
    channelmap,
    context::introspect::{SinkInfo, SinkInputInfo},
    def, format,
    proplist::Proplist,
    sample,
    time::MicroSeconds,
    volume::{ChannelVolumes, Volume},
};

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
    /// Length of queued audio in the output buffer.
    pub latency: MicroSeconds,
    /// Driver name.
    pub driver: Option<String>,
    /// Flags.
    pub flags: def::SinkFlagSet,
    /// Property list.
    pub proplist: Proplist,
    /// The latency this device has been configured to.
    pub configured_latency: MicroSeconds,
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

impl<'a> From<&SinkInfo<'a>> for SinkInformation {
    fn from(value: &SinkInfo<'a>) -> Self {
        let name = value.name.as_ref().map(|x| x.to_string());
        let driver = value.driver.as_ref().map(|x| x.to_string());
        let description = value.description.as_ref().map(|x| x.to_string());
        let monitor_source_name = value.monitor_source_name.as_ref().map(|x| x.to_string());

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
            latency: value.latency.clone(),
            driver,
            flags: value.flags.clone(),
            proplist: value.proplist.clone(),
            configured_latency: value.configured_latency.clone(),
            base_volume: value.base_volume.clone(),
            state: value.state.clone(),
            n_volume_steps: value.n_volume_steps.clone(),
            card: value.card.clone(),
            formats: value.formats.clone(),
        }
    }
}
