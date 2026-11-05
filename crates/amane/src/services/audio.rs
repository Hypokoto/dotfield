mod pulse;

use crate::Service;
use crate::services::worker;

use pulse::Device;
pub use pulse::{SinkInfo, SinkType};

#[derive(Default)]
pub struct Audio {
    // 0 to 100, for the default output
    volume: u8,

    muted: bool,

    // 0 to 100, for the default microphone
    microphone_volume: u8,

    microphone_muted: bool,

    sinks: Vec<SinkInfo>,
}

// event-driven, the sound server announces every volume change
impl Service for Audio {
    fn new() -> Self {
        let mut audio = Self::default();

        audio.update();

        audio
    }

    fn update(&mut self) -> bool {
        let before_levels = (
            self.volume,
            self.muted,
            self.microphone_volume,
            self.microphone_muted,
        );

        let output = pulse::read(Device::Output);
        let input = pulse::read(Device::Input);
        let current_sinks = pulse::read_sinks();

        self.volume = output.volume;
        self.muted = output.muted;

        self.microphone_volume = input.volume;
        self.microphone_muted = input.muted;

        let levels_changed = (
            self.volume,
            self.muted,
            self.microphone_volume,
            self.microphone_muted,
        ) != before_levels;

        let sinks_changed = self.sinks != current_sinks;

        if sinks_changed {
            self.sinks = current_sinks;
        }

        levels_changed || sinks_changed
    }

    // the sound server also announces changes to devices nothing shows
    fn listen() {
        pulse::watch(|| {
            let mut audio = Self::write();

            if !audio.update() {
                audio.quiet();
            }
        });
    }
}

impl Audio {
    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn microphone_volume(&self) -> u8 {
        self.microphone_volume
    }

    pub fn microphone_muted(&self) -> bool {
        self.microphone_muted
    }

    pub fn sinks(&self) -> &[SinkInfo] {
        &self.sinks
    }

    pub fn default_sink(&self) -> Option<&SinkInfo> {
        self.sinks.iter().find(|s| s.is_default)
    }

    // anything above 100 is treated as 100
    pub fn set_volume(volume: u8) {
        worker::run(move || pulse::set_volume(Device::Output, volume.min(100)));
    }

    pub fn toggle_mute() {
        worker::run(|| toggle(Device::Output));
    }

    // anything above 100 is treated as 100
    pub fn set_microphone_volume(volume: u8) {
        worker::run(move || pulse::set_volume(Device::Input, volume.min(100)));
    }

    pub fn toggle_microphone_mute() {
        worker::run(|| toggle(Device::Input));
    }

    pub fn set_default_sink(name: impl Into<String>) {
        let name = name.into();
        worker::run(move || pulse::set_default_sink(&name));
    }
}

fn toggle(device: Device) {
    let level = pulse::read(device);

    pulse::set_muted(device, !level.muted);
}
