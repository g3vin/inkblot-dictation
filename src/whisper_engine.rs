use std::path::Path;
use std::sync::Arc;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
};

use crate::error::DictationError;
use crate::types::{TranscriptSegment, TranscriptionResult};

pub struct WhisperEngine {
    model_path: String,
    context: Arc<WhisperContext>,
}

impl WhisperEngine {
    pub fn load(model_path: impl AsRef<Path>) -> Result<Self, DictationError> {
        let model_path_ref = model_path.as_ref();
        let model_path_string = model_path_ref.to_string_lossy().to_string();

        let context = WhisperContext::new_with_params(
            &model_path_string,
            WhisperContextParameters::default(),
        )
        .map_err(|e| DictationError::ModelLoadFailed(e.to_string()))?;

        Ok(Self {
            model_path: model_path_string,
            context: Arc::new(context),
        })
    }

    pub fn model_path(&self) -> &str {
        &self.model_path
    }

    pub fn transcribe_16khz_mono(
        &self,
        samples: &[f32],
        language: Option<&str>,
    ) -> Result<TranscriptionResult, DictationError> {
        if samples.is_empty() {
            return Ok(TranscriptionResult {
                raw: String::new(),
                cleaned: String::new(),
                segments: vec![],
                duration_seconds: 0.0,
            });
        }

        let mut state = self
            .context
            .create_state()
            .map_err(|e| DictationError::TranscriptionFailed(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Good defaults for dictation chunks.
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_no_context(true);

        if let Some(lang) = language {
            params.set_language(Some(lang));
        }

        state
            .full(params, samples)
            .map_err(|e| DictationError::TranscriptionFailed(e.to_string()))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| DictationError::TranscriptionFailed(e.to_string()))?;

        let mut segments = Vec::new();

        for i in 0..num_segments {
            let text = state
                .full_get_segment_text(i)
                .map_err(|e| DictationError::TranscriptionFailed(e.to_string()))?;

            let start_cs = state
                .full_get_segment_t0(i)
                .map_err(|e| DictationError::TranscriptionFailed(e.to_string()))?;

            let end_cs = state
                .full_get_segment_t1(i)
                .map_err(|e| DictationError::TranscriptionFailed(e.to_string()))?;

            segments.push(TranscriptSegment {
                start: start_cs as f32 / 100.0,
                end: end_cs as f32 / 100.0,
                text: text.trim().to_string(),
            });
        }

        let raw = segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(TranscriptionResult {
            raw: raw.clone(),
            cleaned: raw,
            segments,
            duration_seconds: samples.len() as f32 / 16_000.0,
        })
    }
}