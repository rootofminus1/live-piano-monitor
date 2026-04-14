use crate::{alg::{data::{TrainingData, data_to_dict_matrix}, omp::sparse_encode}, core::{KeyboardSpec, Note, Pitch, generate_keys}};

const COEFFICIENT_THRESHOLD: f32 = 0.10;
const MIN_SEMITONE_DISTANCE: i32 = 2;


pub struct Detector {
    pub dict: Vec<Vec<f32>>,
    pub note_map: Vec<Pitch>
}


impl Detector {
    pub fn new() -> Self {
        let td = TrainingData::load("fretdata.bin".as_ref())
            .expect("failed to load training data");

        let dict = data_to_dict_matrix(&td.notes);

        let keys = generate_keys(&KeyboardSpec {
            start_note: Note::C,
            start_octave: 3,
            key_count: dict.len(),
        });

        let note_map = keys
            .into_iter()
            .map(|k| Pitch::new(k.note, k.octave))
            .collect();

        Self { dict, note_map }
    }

    
    pub fn process(&self, fft: Vec<f32>) -> Vec<Pitch> {
        let s = sparse_encode(&fft, &self.dict, 6);

        let mut indexed: Vec<(usize, f32)> =
            s.iter().cloned().enumerate().collect();

        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        indexed.truncate(6);

        let total: f32 = indexed.iter().filter(|(_, c)| *c > 0.0).map(|(_, c)| c).sum();
        if total <= 0.0 {
            return vec![];
        }

        let mut result = Vec::new();
        let mut kept_midi: Vec<i32> = Vec::new();

        for (i, coef) in &indexed {
            let normalized = coef / total;
            if normalized < COEFFICIENT_THRESHOLD {
                break; // sorted descendingso everything after is also too weak
            }

            let Some(pitch) = self.note_map.get(*i) else { continue };

            let midi = pitch.octave * 12 + pitch.note as i32;
            if kept_midi.iter().any(|&m| (midi - m).abs() < MIN_SEMITONE_DISTANCE) {
                continue;
            }

            result.push(*pitch);
            kept_midi.push(midi);
        }

        result
    }
}