use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::cleanup::clean_transcript;
use crate::error::DictationError;
use crate::resample::{interleaved_to_mono, linear_resample_mono};
use crate::types::{
    DictationErrorPayload, DictationEvent, DictationEventCallback, DictationEventPayload,
    LiveDictationOptions,
};
use crate::vad::SimpleVad;
use crate::whisper_engine::WhisperEngine;

const WHISPER_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug)]
struct AudioPacket {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: usize,
}

#[derive(Debug)]
struct AudioChunk {
    samples_16k_mono: Vec<f32>,
    duration_seconds: f32,
    is_final: bool,
}

pub struct DictationSession {
    pub session_id: String,
    stop_flag: Arc<AtomicBool>,
    stream: Option<Stream>,
}

impl DictationSession {
    pub fn start(
        engine: Arc<WhisperEngine>,
        options: LiveDictationOptions,
        callback: DictationEventCallback,
    ) -> Result<Self, DictationError> {
        let session_id = Uuid::new_v4().to_string();
        let stop_flag = Arc::new(AtomicBool::new(false));

        let (audio_tx, audio_rx) = mpsc::channel::<AudioPacket>(64);
        let (chunk_tx, chunk_rx) = mpsc::channel::<AudioChunk>(16);

        let stream = build_input_stream(audio_tx)?;

        stream
            .play()
            .map_err(|e| DictationError::MicrophoneStartFailed(e.to_string()))?;

        callback(DictationEvent::Started(DictationEventPayload {
            session_id: session_id.clone(),
            text: String::new(),
            raw: None,
            cleaned: None,
            is_final: false,
            duration_seconds: None,
        }));

        spawn_chunk_builder(
            session_id.clone(),
            stop_flag.clone(),
            options.clone(),
            audio_rx,
            chunk_tx,
            callback.clone(),
        );

        spawn_transcription_worker(
            session_id.clone(),
            engine,
            options,
            chunk_rx,
            callback.clone(),
        );

        Ok(Self {
            session_id,
            stop_flag,
            stream: Some(stream),
        })
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);

        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
    }
}

fn build_input_stream(audio_tx: mpsc::Sender<AudioPacket>) -> Result<Stream, DictationError> {
    let host = cpal::default_host();

    let device = host.default_input_device().ok_or_else(|| {
        DictationError::MicrophoneInitFailed("No default input device found".to_string())
    })?;

    let supported_config = device
        .default_input_config()
        .map_err(|e| DictationError::MicrophoneInitFailed(e.to_string()))?;

    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    let sample_rate = config.sample_rate;
    let channels = config.channels as usize;

    let err_fn = move |err| {
        eprintln!("CPAL audio stream error: {err}");
    };

    match sample_format {
        SampleFormat::F32 => build_typed_input_stream::<f32>(
            &device,
            &config,
            sample_rate,
            channels,
            audio_tx,
            err_fn,
        ),
        SampleFormat::I16 => build_typed_input_stream::<i16>(
            &device,
            &config,
            sample_rate,
            channels,
            audio_tx,
            err_fn,
        ),
        SampleFormat::U16 => build_typed_input_stream::<u16>(
            &device,
            &config,
            sample_rate,
            channels,
            audio_tx,
            err_fn,
        ),
        other => Err(DictationError::MicrophoneInitFailed(format!(
            "Unsupported sample format: {other:?}"
        ))),
    }
}

fn build_typed_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_rate: u32,
    channels: usize,
    audio_tx: mpsc::Sender<AudioPacket>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, DictationError>
where
    T: Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let tx = audio_tx;

    device
        .build_input_stream(
            config,
            move |data: &[T], _info: &cpal::InputCallbackInfo| {
                let samples: Vec<f32> = data.iter().map(|s| f32::from_sample(*s)).collect();

                // Do not block the real-time audio thread.
                let _ = tx.try_send(AudioPacket {
                    samples,
                    sample_rate,
                    channels,
                });
            },
            err_fn,
            None,
        )
        .map_err(|e| DictationError::MicrophoneInitFailed(e.to_string()))
}

fn spawn_chunk_builder(
    session_id: String,
    stop_flag: Arc<AtomicBool>,
    options: LiveDictationOptions,
    mut audio_rx: mpsc::Receiver<AudioPacket>,
    chunk_tx: mpsc::Sender<AudioChunk>,
    callback: DictationEventCallback,
) {
    tokio::spawn(async move {
        let silence_ms = options.silence_ms.unwrap_or(900);
        let min_chunk_ms = options.min_chunk_ms.unwrap_or(1200);
        let max_chunk_ms = options.max_chunk_ms.unwrap_or(12_000);
        let partial_interval_ms = options.partial_interval_ms.unwrap_or(1500);
        let silence_threshold = options.silence_rms_threshold.unwrap_or(0.012);

        let vad = SimpleVad::new(silence_threshold);

        let mut current_chunk: Vec<f32> = Vec::new();
        let mut speech_started = false;

        let mut last_speech_at = Instant::now();
        let mut chunk_started_at = Instant::now();
        let mut last_partial_at = Instant::now();

        while !stop_flag.load(Ordering::SeqCst) {
            let Some(packet) = audio_rx.recv().await else {
                break;
            };

            let mono = interleaved_to_mono(&packet.samples, packet.channels);
            let samples_16k =
                linear_resample_mono(&mono, packet.sample_rate, WHISPER_SAMPLE_RATE);

            let is_speech = vad.is_speech(&samples_16k);

            if is_speech {
                if !speech_started {
                    speech_started = true;
                    chunk_started_at = Instant::now();
                    last_partial_at = Instant::now();
                    current_chunk.clear();
                }

                last_speech_at = Instant::now();
                current_chunk.extend_from_slice(&samples_16k);
            } else if speech_started {
                current_chunk.extend_from_slice(&samples_16k);
            }

            if !speech_started {
                continue;
            }

            let elapsed_chunk_ms = chunk_started_at.elapsed().as_millis() as u64;
            let elapsed_silence_ms = last_speech_at.elapsed().as_millis() as u64;
            let elapsed_partial_ms = last_partial_at.elapsed().as_millis() as u64;

            let has_min_chunk = elapsed_chunk_ms >= min_chunk_ms;
            let hit_silence = has_min_chunk && elapsed_silence_ms >= silence_ms;
            let hit_max = elapsed_chunk_ms >= max_chunk_ms;
            let should_partial =
                has_min_chunk && elapsed_partial_ms >= partial_interval_ms && !current_chunk.is_empty();

            if should_partial {
                let partial = current_chunk.clone();
                last_partial_at = Instant::now();

                let _ = chunk_tx
                    .send(AudioChunk {
                        duration_seconds: partial.len() as f32 / WHISPER_SAMPLE_RATE as f32,
                        samples_16k_mono: partial,
                        is_final: false,
                    })
                    .await;
            }

            if hit_silence || hit_max {
                let final_samples = std::mem::take(&mut current_chunk);

                if !final_samples.is_empty() {
                    let _ = chunk_tx
                        .send(AudioChunk {
                            duration_seconds: final_samples.len() as f32
                                / WHISPER_SAMPLE_RATE as f32,
                            samples_16k_mono: final_samples,
                            is_final: true,
                        })
                        .await;
                }

                speech_started = false;
                last_partial_at = Instant::now();
            }
        }

        if !current_chunk.is_empty() {
            let _ = chunk_tx
                .send(AudioChunk {
                    duration_seconds: current_chunk.len() as f32 / WHISPER_SAMPLE_RATE as f32,
                    samples_16k_mono: current_chunk,
                    is_final: true,
                })
                .await;
        }

        callback(DictationEvent::Stopped(DictationEventPayload {
            session_id,
            text: String::new(),
            raw: None,
            cleaned: None,
            is_final: true,
            duration_seconds: None,
        }));
    });
}

fn spawn_transcription_worker(
    session_id: String,
    engine: Arc<WhisperEngine>,
    options: LiveDictationOptions,
    mut chunk_rx: mpsc::Receiver<AudioChunk>,
    callback: DictationEventCallback,
) {
    tokio::spawn(async move {
        while let Some(chunk) = chunk_rx.recv().await {
            let session_id = session_id.clone();
            let engine = engine.clone();
            let language = options.language.clone();
            let cleanup = options.cleanup.unwrap_or(true);
            let callback = callback.clone();

            tokio::task::spawn_blocking(move || {
                let result =
                    engine.transcribe_16khz_mono(&chunk.samples_16k_mono, language.as_deref());

                match result {
                    Ok(mut transcript) => {
                        let cleaned = if cleanup {
                            clean_transcript(&transcript.raw)
                        } else {
                            transcript.raw.clone()
                        };

                        transcript.cleaned = cleaned.clone();

                        let text = if cleanup {
                            cleaned.clone()
                        } else {
                            transcript.raw.clone()
                        };

                        let payload = DictationEventPayload {
                            session_id,
                            text,
                            raw: Some(transcript.raw),
                            cleaned: Some(cleaned),
                            is_final: chunk.is_final,
                            duration_seconds: Some(chunk.duration_seconds),
                        };

                        if chunk.is_final {
                            callback(DictationEvent::Final(payload));
                        } else {
                            callback(DictationEvent::Partial(payload));
                        }
                    }
                    Err(err) => {
                        callback(DictationEvent::Error(DictationErrorPayload {
                            session_id: Some(session_id),
                            message: err.to_string(),
                        }));
                    }
                }
            });
        }
    });
}