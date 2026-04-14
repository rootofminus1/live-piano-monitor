use crate::core::Note;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pitch {
    pub note: Note,
    pub octave: i32,
}

impl Pitch {
    pub fn new(note: Note, octave: i32) -> Self {
        Self { note, octave }
    }
}

// might not be needed anymore
pub fn freq_to_pitch(freq: f32) -> Option<Pitch> {
    if freq <= 0.0 { return None; }

    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    let midi = midi.round() as i32;

    let note_index = midi.rem_euclid(12);
    let octave = midi / 12 - 1;

    // Some((Note::all_notes()[note_index as usize], octave))
    Some(Pitch::new(Note::all_notes()[note_index as usize], octave))
}