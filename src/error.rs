use thiserror::Error;

#[derive(Debug, Error)]
pub enum DictationError {
    #[error("No dictation model is loaded")]
    ModelNotLoaded,

    #[error("A dictation session is already running")]
    SessionAlreadyRunning,

    #[error("No dictation session is currently running")]
    NoSessionRunning,

    #[error("Failed to load Whisper model: {0}")]
    ModelLoadFailed(String),

    #[error("Failed to initialize microphone input: {0}")]
    MicrophoneInitFailed(String),

    #[error("Failed to start microphone stream: {0}")]
    MicrophoneStartFailed(String),

    #[error("Audio stream error: {0}")]
    AudioStreamError(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("Internal channel closed")]
    ChannelClosed,
}

impl From<DictationError> for String {
    fn from(value: DictationError) -> Self {
        value.to_string()
    }
}