//! Data Structures for Pulse Audio
use pulse::{
    context::introspect::SinkInputInfo,
    channelmap,
    format,
    proplist::Proplist,
    sample,
    time::MicroSeconds,
    volume::ChannelVolumes,
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
