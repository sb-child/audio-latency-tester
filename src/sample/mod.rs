use std::sync::Arc;

pub mod fileloader;

#[derive(Clone)]
pub struct Sample {
    data: Arc<Vec<f32>>,
    rate: u32,
}

impl Sample {
    pub fn new(data: Arc<Vec<f32>>, rate: u32) -> Self {
        Self { data, rate }
    }

    pub fn get_data(&self) -> Arc<Vec<f32>> {
        self.data.clone()
    }

    pub fn get_rate(&self) -> u32 {
        self.rate
    }
}
