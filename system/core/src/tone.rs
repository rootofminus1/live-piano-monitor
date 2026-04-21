use crate::Note;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Tone {
    pub note: Note,
    pub octave: i32,
}

impl Tone {
    pub fn new(note: Note, octave: i32) -> Self {
        Self { note, octave }
    }

    pub fn from_freq(freq: f32) -> Option<Self> {
        if freq <= 0.0 { return None; }

        let midi = 69.0 + 12.0 * (freq / 440.0).log2();
        let midi = midi.round() as i32;

        let note_index = midi.rem_euclid(12);
        let octave = midi / 12 - 1;

        // Some((Tone::all_notes()[note_index as usize], octave))
        Some(Tone::new(Note::all_notes()[note_index as usize], octave))
    }
}