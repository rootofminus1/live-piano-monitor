use core::Tone;

use serde::{Deserialize, Serialize};


pub trait PitchProcessor: Send {
    fn process_block(&mut self, block: &[f32]) -> Option<Vec<Tone>>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum DetectionMode {
    /// LARS polyphonic with training required.
    #[default]
    Polyphonic,
    /// YIN monophonic
    Monophonic,
}