use crate::Tone;

use super::Note;

// #[derive(Debug, Clone)]
// pub struct KeyInfo {
//     pub note: Note,
//     pub octave: i32,
// }

#[derive(Debug, Clone)]
pub struct KeyboardSpec {
    pub start_tone: Tone,
    pub key_count: usize,
}

impl KeyboardSpec {
    pub fn from_octaves(
        start_tone: Tone,
        octaves: usize,
    ) -> Self {
        Self {
            start_tone,
            key_count: octaves * 12 + 1,  // this might not work for when we start from some cursed key rather than a C, TODO: test
        }
    }

    pub fn piano_88() -> Self {
        Self {
            start_tone: Tone { note: Note::A, octave: 0 },
            key_count: 88,
        }
    }

    pub fn piano_smaller() -> Self {
        Self {
            start_tone: Tone { note: Note::C, octave: 2 },
            key_count: 61,
        }
    }
}

pub fn generate_keys(spec: &KeyboardSpec) -> Vec<Tone> {
    let scale = Note::all_notes();

    let mut keys = Vec::with_capacity(spec.key_count);
    let mut octave = spec.start_tone.octave;
    let mut index = scale.iter().position(|&n| n == spec.start_tone.note).unwrap();  // TODO: err handle

    for _ in 0..spec.key_count {
        let note = scale[index];
        keys.push(Tone { note, octave });

        index += 1;
        if index == 12 {
            index = 0;
            octave += 1;
        }
    }

    keys
}