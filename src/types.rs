use serde::{Deserialize, Serialize};

use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadModelOptions {
    pub model_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub model_path: String,
    pub loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDictationOptions {
    /// Optional language hint. Use Some("en") for English-only dictation.
    pub language: Option<String>,

    /// Emit rough partial previews every N milliseconds.
    pub partial_interval_ms: Option<u64>,

    /// Finalize a chunk after this much silence.
    pub silence_ms: Option<u64>,

    /// Minimum speech chunk before transcribing.
    pub min_chunk_ms: Option<u64>,

    /// Maximum speech chunk before forcing finalization.
    pub max_chunk_ms: Option<u64>,

    /// RMS threshold for simple silence detection.
    pub silence_rms_threshold: Option<f32>,

    /// Whether to clean the transcript.
    pub cleanup: Option<bool>,
}

impl Default for LiveDictationOptions {
    fn default() -> Self {
        Self {
            language: Some("en".to_string()),
            partial_interval_ms: Some(1500),
            silence_ms: Some(900),
            min_chunk_ms: Some(1200),
            max_chunk_ms: Some(12_000),
            silence_rms_threshold: Some(0.012),
            cleanup: Some(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationStatus {
    pub model_loaded: bool,
    pub session_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub start: f32,
    pub end: f32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub raw: String,
    pub cleaned: String,
    pub segments: Vec<TranscriptSegment>,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationEventPayload {
    pub session_id: String,
    pub text: String,
    pub raw: Option<String>,
    pub cleaned: Option<String>,
    pub is_final: bool,
    pub duration_seconds: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationErrorPayload {
    pub session_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum DictationEvent {
    Started(DictationEventPayload),
    Partial(DictationEventPayload),
    Final(DictationEventPayload),
    Stopped(DictationEventPayload),
    Error(DictationErrorPayload),
}

pub type DictationEventCallback = Arc<dyn Fn(DictationEvent) + Send + Sync + 'static>;