use std::sync::Arc;

use crate::error::DictationError;
use crate::session::DictationSession;
use crate::types::{
    DictationEventCallback, DictationStatus, LiveDictationOptions, ModelInfo,
};
use crate::whisper_engine::WhisperEngine;

#[derive(Default)]
pub struct DictationService {
    engine: Option<Arc<WhisperEngine>>,
    session: Option<DictationSession>,
}

impl DictationService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_model(
        &mut self,
        model_path: impl AsRef<str>,
    ) -> Result<ModelInfo, DictationError> {
        let engine = WhisperEngine::load(model_path.as_ref())?;

        let info = ModelInfo {
            model_path: engine.model_path().to_string(),
            loaded: true,
        };

        self.engine = Some(Arc::new(engine));

        Ok(info)
    }

    pub fn unload_model(&mut self) {
        self.cancel_session();
        self.engine = None;
    }

    pub fn start_session(
        &mut self,
        options: LiveDictationOptions,
        callback: DictationEventCallback,
    ) -> Result<String, DictationError> {
        if self.session.is_some() {
            return Err(DictationError::SessionAlreadyRunning);
        }

        let engine = self
            .engine
            .clone()
            .ok_or(DictationError::ModelNotLoaded)?;

        let session = DictationSession::start(engine, options, callback)?;
        let session_id = session.session_id.clone();

        self.session = Some(session);

        Ok(session_id)
    }

    pub fn stop_session(&mut self) -> Result<(), DictationError> {
        let Some(session) = self.session.as_mut() else {
            return Err(DictationError::NoSessionRunning);
        };

        session.stop();
        self.session = None;

        Ok(())
    }

    pub fn cancel_session(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.stop();
        }

        self.session = None;
    }

    pub fn status(&self) -> DictationStatus {
        DictationStatus {
            model_loaded: self.engine.is_some(),
            session_running: self.session.is_some(),
        }
    }
}