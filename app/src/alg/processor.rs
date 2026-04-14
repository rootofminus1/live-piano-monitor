use crate::core::Pitch;
use bevy::ecs::resource::Resource;

pub trait PitchProcessor: Send {
    fn process_block(&mut self, block: &[f32]) -> Option<Vec<Pitch>>;
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DetectionMode {
    /// LARS polyphonic with training required.
    #[default]
    Polyphonic,
    /// YIN monophonic
    Monophonic,
}