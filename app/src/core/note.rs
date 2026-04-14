
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display, Hash)]
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

