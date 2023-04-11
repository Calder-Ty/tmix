//! Data manager For Application
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::pulse_api::SinkInputInformation;

pub struct DataManager {
    rx: Receiver<SinkInputInformation>,
    data: HashMap<u32, SinkInputInformation>,
}

impl DataManager {
    pub fn new(rx: Receiver<SinkInputInformation>) -> Self {
        Self {
            rx,
            data: HashMap::default(),
        }
    }

    pub fn get(&self, k: &u32) -> Option<&SinkInputInformation> {
        self.data.get(k)
    }

    pub fn values(&self) -> Values<'_, u32, SinkInputInformation> {
        self.data.values()
    }

    pub fn update(&mut self) {
        if let Ok(data) =  self.rx.recv_timeout(Duration::from_millis(5)) {
            self.data.insert(data.index, data);
        }
    }
}
