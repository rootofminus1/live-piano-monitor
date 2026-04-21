use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::dsp::CROP_SIZE;


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TrainingData {
    pub notes: Vec<Vec<Vec<f32>>>, // [note_idx][frame_idx][bin]
}

impl TrainingData {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        bincode::deserialize(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
    
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let bytes = bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
}


// TODO: maybe move this somewhere?
pub fn data_to_dict_matrix(data: &[Vec<Vec<f32>>]) -> Vec<Vec<f32>> {
    data.iter().map(|frames| {
        if frames.is_empty() { return vec![0.0f32; CROP_SIZE]; }
        let mut avg = vec![0.0f32; CROP_SIZE];

        for frame in frames {
            for (a, f) in avg.iter_mut().zip(frame.iter()) { *a += f; }
        }
        
        let n = frames.len() as f32;
        for a in avg.iter_mut() { *a /= n; }
        let norm: f32 = avg.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm < 1e-12 { avg } else { avg.iter().map(|x| x / norm).collect() }
    }).collect()
}