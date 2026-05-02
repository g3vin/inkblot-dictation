//! Inkblot Dictation
//!
//! Offline dictation pipeline using local microphone capture,
//! VAD-based chunking, and Whisper transcription.

pub mod cleanup;
pub mod error;
pub mod resample;
pub mod service;
pub mod session;
pub mod types;
pub mod vad;
pub mod whisper_engine;

pub use error::DictationError;
pub use service::DictationService;
pub use types::{
    DictationErrorPayload, DictationEvent, DictationEventCallback, DictationEventPayload,
    DictationStatus, LiveDictationOptions, LoadModelOptions, ModelInfo, TranscriptSegment,
    TranscriptionResult,
};
pub use whisper_engine::WhisperEngine;