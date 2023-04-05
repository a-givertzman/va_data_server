struct AverageFilter {
    len: usize,
    values: Vec<f32>,
}

impl AverageFilter {
    ///
    pub fn new(len: usize) -> Self {
        Self {
            len,
            values: vec![0.0; len],
        }
    }
    ///
    pub fn add(&mut self, value: f32) {
        self.values.push(value);
        if self.values.len() > self.len {
            self.values.remove(0);
        }
    }
    ///
    pub fn value(&self) -> f32 {
        self.values.iter().sum::<f32>() / (self.len as f32)
    }
}