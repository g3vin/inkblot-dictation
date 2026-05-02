use std::env;
use std::sync::Arc;

use inkblot_dictation::{
    DictationEvent,
    DictationEventCallback,
    DictationService,
    LiveDictationOptions,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage:");
        eprintln!("  cargo run --example live_dictation -- <model.bin>");
        std::process::exit(1);
    }

    let model_path = &args[1];

    println!("Loading model: {model_path}");

    let mut service = DictationService::new();
    service.load_model(model_path)?;

    println!("Starting live dictation.");
    println!("Speak into your microphone. Press Ctrl+C to stop.");
    println!();

    let callback: DictationEventCallback = Arc::new(move |event| {
        match event {
            DictationEvent::Started(payload) => {
                println!("[started] session={}", payload.session_id);
            }
            DictationEvent::Partial(payload) => {
                println!("[partial] {}", payload.text);
            }
            DictationEvent::Final(payload) => {
                println!("[final] {}", payload.text);
            }
            DictationEvent::Stopped(payload) => {
                println!("[stopped] session={}", payload.session_id);
            }
            DictationEvent::Error(payload) => {
                eprintln!("[error] {}", payload.message);
            }
        }
    });

    let options = LiveDictationOptions {
        language: Some("en".to_string()),
        partial_interval_ms: Some(1500),
        silence_ms: Some(900),
        min_chunk_ms: Some(1200),
        max_chunk_ms: Some(12_000),
        silence_rms_threshold: Some(0.012),
        cleanup: Some(true),
    };

    let session_id = service.start_session(options, callback)?;

    println!("Session ID: {session_id}");

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}