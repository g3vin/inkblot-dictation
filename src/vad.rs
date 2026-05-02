#[derive(Debug, Clone)]
pub struct SimpleVad {
    silence_rms_threshold: f32,
}

impl SimpleVad {
    pub fn new(silence_rms_threshold: f32) -> Self {
        Self {
            silence_rms_threshold,
        }
    }

    pub fn is_speech(&self, samples: &[f32]) -> bool {
        if samples.is_empty() {
            return false;
        }

        rms(samples) >= self.silence_rms_threshold
    }
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}