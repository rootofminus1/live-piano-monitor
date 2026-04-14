use crate::{alg::{data::{TrainingData, data_to_dict_matrix}, dsp::{FFT_ACCUMULATE_BLOCKS, OFFSET_THRESHOLD_DB, ONSET_THRESHOLD_DB, TOTAL_SIZE, compute_fft, rms_db}, processor::PitchProcessor, sparse::sparse_encode}, core::{KeyboardSpec, Note, Pitch, generate_keys}};


const COEFFICIENT_THRESHOLD: f32 = 0.10;
const MIN_SEMITONE_DISTANCE: i32 = 2;


pub struct LarsProcessor {
    dict: Vec<Vec<f32>>,
    note_map: Vec<Pitch>,

    note_on: bool,
    ring: Vec<f32>,
    write_pos: usize,
    block_count: usize,
}

impl LarsProcessor {
    pub fn new() -> Self {
        let td = TrainingData::load("fretdata.bin".as_ref())
            .expect("failed to load training data");
        let dict = data_to_dict_matrix(&td.notes);

        let keys = generate_keys(&KeyboardSpec {
            start_note: Note::C,
            start_octave: 3,
            key_count: dict.len(),
        });
        let note_map = keys.into_iter().map(|k| Pitch::new(k.note, k.octave)).collect();

        Self {
            dict,
            note_map,
            note_on: false,
            ring: vec![0.0f32; TOTAL_SIZE],
            write_pos: 0,
            block_count: 0,
        }
    }

    fn detect(&self, fft: Vec<f32>) -> Vec<Pitch> {
        let s = sparse_encode(&fft, &self.dict, 6);

        let mut indexed: Vec<(usize, f32)> = s.into_iter().enumerate().collect();
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
                break;
            }

            let Some(pitch) = self.note_map.get(*i) else { continue };

            // Rough MIDI number for distance check: octave * 12 + note_index
            let note_idx = Note::all_notes()
                .iter()
                .position(|&n| n == pitch.note)
                .unwrap_or(0) as i32;
            let midi = pitch.octave * 12 + note_idx;

            if kept_midi.iter().any(|&m| (midi - m).abs() < MIN_SEMITONE_DISTANCE) {
                continue;
            }

            result.push(*pitch);
            kept_midi.push(midi);
        }

        result
    }
}

impl PitchProcessor for LarsProcessor {
    fn process_block(&mut self, block: &[f32]) -> Option<Vec<Pitch>> {
        let db = rms_db(block);

        if !self.note_on && db >= ONSET_THRESHOLD_DB {
            self.note_on = true;
            self.ring.iter_mut().for_each(|x| *x = 0.0);
            self.write_pos = 0;
            self.block_count = 0;
        } else if self.note_on && db < OFFSET_THRESHOLD_DB {
            self.note_on = false;
            return Some(vec![]);  // silence
        }

        if !self.note_on {
            return None;
        }

        for &s in block {
            self.ring[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % TOTAL_SIZE;
        }

        self.block_count += 1;
        if self.block_count < FFT_ACCUMULATE_BLOCKS {
            return None;
        }
        self.block_count = 0;

        let mut buf = Vec::with_capacity(TOTAL_SIZE);
        buf.extend_from_slice(&self.ring[self.write_pos..]);
        buf.extend_from_slice(&self.ring[..self.write_pos]);

        compute_fft(&buf).map(|fft| self.detect(fft))
    }
}