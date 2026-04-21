
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display, Hash, serde::Serialize, serde::Deserialize)]
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
        matches!(self, Self::Cs | Self::Ds | Self::Fs | Self::Gs | Self::As)
    }

    pub fn is_white(&self) -> bool {
        !self.is_black()
    }

    pub fn all_notes() -> [Self; 12] {
        [
            Self::C, Self::Cs, Self::D, Self::Ds, Self::E,
            Self::F, Self::Fs, Self::G, Self::Gs,
            Self::A, Self::As, Self::B,
        ]
    }
}

