# inkblot-dictation

On-device live dictation pipline for tarui + React apps using:

- `cpal` for microphone capture
- simple VAD / silence detection for chunking
- `whisper-rs` / `whisper.cpp` for local speech-to-text
- callback-based transcript events for app integration

This crate is designed to be used by a Tauri app, CLI, or any Rust application without depending directly on Tauri

---

## Features

- Offline speech recognition
- Live microphone capture
- Silence-based chunk finalization
- Partial and final transcript events
- Whisper model loaded once and reused
- Optional transcript cleanup:
  - spoken punctuation
  - `new line`
  - `new paragraph`
  - capitalization
  - standalone `I`

---

## Requirements

- Rust / Cargo
- CMake
- A Whisper `ggml` model file
- macOS: Xcode Command Line Tools
- Optional: FFmpeg for converting audio samples

---

## Quickstart

```bash
brew install cmake ffmpeg
mkdir -p models
curl -L -o models/ggml-small.en.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin
```
and then run the crate:
```bash
cargo run --example live_dictation -- models/ggml-small.en.bin
```
or if you're using MacOS:
```bash
cargo run --features metal --example live_dictation -- models/ggml-small.en.bin
```
