mod midi_backend;
use midi_backend::{MidiBackend, MidiPortInfo};

#[cfg(feature = "synth")]
mod synth_backend;

#[cfg(feature = "synth")]
use synth_backend::SynthBackend;

use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use midi_file::midly::{num::u4, MidiMessage};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OutputDescriptor {
    #[cfg(feature = "synth")]
    Synth(Option<PathBuf>),
    MidiOut(MidiPortInfo),
    DummyOutput,
}

impl Display for OutputDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "synth")]
            OutputDescriptor::Synth(_) => write!(f, "Buildin Synth"),
            OutputDescriptor::MidiOut(info) => write!(f, "{}", info),
            OutputDescriptor::DummyOutput => write!(f, "No Output"),
        }
    }
}

pub trait OutputConnection {
    fn midi_event(&mut self, channel: u4, msg: MidiMessage);
    fn stop_all(&mut self);
}

struct DummyOutput {}
impl OutputConnection for DummyOutput {
    fn midi_event(&mut self, _channel: u4, _msg: MidiMessage) {}
    fn stop_all(&mut self) {}
}

pub struct OutputManager {
    #[cfg(feature = "synth")]
    synth_backend: Option<SynthBackend>,
    midi_backend: Option<MidiBackend>,

    output_connection: (OutputDescriptor, Box<dyn OutputConnection>),
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputManager {
    pub fn new() -> Self {
        #[cfg(feature = "synth")]
        let synth_backend = match SynthBackend::new() {
            Ok(synth_backend) => Some(synth_backend),
            Err(err) => {
                log::error!("{:?}", err);
                None
            }
        };

        let midi_backend = match MidiBackend::new() {
            Ok(midi_device_manager) => Some(midi_device_manager),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        };

        Self {
            #[cfg(feature = "synth")]
            synth_backend,
            midi_backend,

            output_connection: (OutputDescriptor::DummyOutput, Box::new(DummyOutput {})),
        }
    }

    pub fn outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();

        #[cfg(feature = "synth")]
        if let Some(synth) = &self.synth_backend {
            outs.append(&mut synth.get_outputs());
        }
        if let Some(midi) = &self.midi_backend {
            outs.append(&mut midi.get_outputs());
        }

        outs.push(OutputDescriptor::DummyOutput);

        outs
    }

    pub fn connect(&mut self, desc: OutputDescriptor) {
        if desc != self.output_connection.0 {
            match desc {
                #[cfg(feature = "synth")]
                OutputDescriptor::Synth(ref font) => {
                    if let Some(ref mut synth) = self.synth_backend {
                        if let Some(font) = font.clone() {
                            self.output_connection =
                                (desc, Box::new(synth.new_output_connection(&font)));
                        } else if let Some(path) = crate::utils::resources::default_sf2() {
                            if path.exists() {
                                self.output_connection =
                                    (desc, Box::new(synth.new_output_connection(&path)));
                            }
                        }
                    }
                }
                OutputDescriptor::MidiOut(ref info) => {
                    if let Some(conn) = MidiBackend::new_output_connection(info) {
                        self.output_connection = (desc, Box::new(conn));
                    }
                }
                OutputDescriptor::DummyOutput => {
                    self.output_connection = (desc, Box::new(DummyOutput {}));
                }
            }
        }
    }

    pub fn midi_event(&mut self, channel: u4, msg: MidiMessage) {
        self.output_connection.1.midi_event(channel, msg);
    }

    pub fn stop_all(&mut self) {
        self.output_connection.1.stop_all();
    }
}
