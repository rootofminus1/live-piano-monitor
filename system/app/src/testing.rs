#[cfg(test)]
mod tests {
    use core::*;

    #[test]
    fn test_generate_keys() {
        // generate keys for a small piano
        let spec = KeyboardSpec::piano_smaller();
        let keys = generate_keys(&spec);
        assert_eq!(keys.len(), spec.key_count);

        // generate keys for a standard 88-key piano
        let spec = KeyboardSpec::piano_88();
        let keys = generate_keys(&spec);
        assert_eq!(keys.len(), spec.key_count);

        // generate keys for a custom spec
        let spec = KeyboardSpec {
            start_tone: Tone::new(Note::C, 4),
            key_count: 24,
        };
        let keys = generate_keys(&spec);
        assert_eq!(keys.len(), spec.key_count);

        // generate keys for a spec starting with a black key
        let spec = KeyboardSpec {
            start_tone: Tone::new(Note::Cs, 4),
            key_count: 12,
        };
        let keys = generate_keys(&spec);
        assert_eq!(keys.len(), spec.key_count);

        // generate keys for a specification with a single key
        let spec = KeyboardSpec {
            start_tone: Tone::new(Note::C, 4),
            key_count: 1,
        };
        let keys = generate_keys(&spec);
        assert_eq!(keys.len(), spec.key_count);
    }

    #[test]
    fn test_keyboard_spec_from_octaves() {
        let spec = KeyboardSpec::from_octaves(Tone::new(Note::C, 4), 2);
        assert_eq!(spec.start_tone.note, Note::C);
        assert_eq!(spec.start_tone.octave, 4);
        assert_eq!(spec.key_count, 25);
    }

    #[test]
    fn test_keyboard_spec_piano_88() {
        let spec = KeyboardSpec::piano_88();
        assert_eq!(spec.start_tone.note, Note::A);
        assert_eq!(spec.start_tone.octave, 0);
        assert_eq!(spec.key_count, 88);
    }

    #[test]
    fn test_keyboard_spec_piano_smaller() {
        let spec = KeyboardSpec::piano_smaller();
        assert_eq!(spec.start_tone.note, Note::C);
        assert_eq!(spec.start_tone.octave, 3);
        assert_eq!(spec.key_count, 61);
    }
}