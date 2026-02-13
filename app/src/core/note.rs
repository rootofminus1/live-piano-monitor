use strum_macros::Display;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Note {
    C, Cs,
    D, Ds,
    E,
    F, Fs,
    G, Gs,
    A, As,
    B,
}

impl Note {
    pub fn is_black(&self) -> bool {
        matches!(self, Note::Cs | Note::Ds | Note::Fs | Note::Gs | Note::As)
    }

    pub fn is_white(&self) -> bool {
        !self.is_black()
    }

    pub fn all_notes() -> [Self; 12] {
        [
            Note::C, Note::Cs, Note::D, Note::Ds, Note::E,
            Note::F, Note::Fs, Note::G, Note::Gs,
            Note::A, Note::As, Note::B,
        ]
    }
}

// TODO: maybe move this somewhere
pub fn freq_to_note(freq: f32) -> Option<(Note, i32)> {
    if freq <= 0.0 { return None; }

    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    let midi = midi.round() as i32;

    let note_index = midi.rem_euclid(12);
    let octave = midi / 12 - 1;

    Some((Note::all_notes()[note_index as usize], octave))
}