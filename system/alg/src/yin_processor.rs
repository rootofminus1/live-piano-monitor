use core::Tone;

use yin::Yin;

use crate::{PitchProcessor, dsp::SR};


pub struct YinProcessor {
    yin: Yin,
    buffer: Vec<f64>,
    buffer_size: usize,
}

impl YinProcessor {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            yin: Yin::init(0.15, 50.0, 1000.0, SR as usize),
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }
}

impl Default for YinProcessor {
    fn default() -> Self {
        Self::new(2048)
    }
}

impl PitchProcessor for YinProcessor {
    fn process_block(&mut self, block: &[f32]) -> Option<Vec<Tone>> {
        // mono assumed
        self.buffer.extend(block.iter().map(|&s| s as f64));

        if self.buffer.len() < self.buffer_size {
            return None;
        }

        let freq = self.yin.estimate_freq(&self.buffer);
        self.buffer.clear();

        if freq.is_finite() && freq > 20.0 && freq < 5000.0 {
            Tone::from_freq(freq as f32).map(|p| vec![p])
        } else {
            Some(vec![])
        }
    }
}