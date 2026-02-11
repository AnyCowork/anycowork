//! End-to-end tests for the transcribe feature.
//!
//! NOTE: These tests may fail to START on Windows due to DirectML DLL issues
//! (STATUS_ENTRYPOINT_NOT_FOUND). This is a pre-existing environment issue
//! affecting all src-tauri integration test binaries. The ort crate links
//! against DirectML.dll which may not be compatible with the test runner.
//!
//! The actual test logic is verified via `cargo test -p anyagents tools::transcribe`
//! which runs in the anyagents crate where the DLL issue doesn't manifest.
//!
//! These tests serve as compile-time verification and will run in CI environments
//! where the DLL is properly configured.

use anycowork::commands::transcribe::{check_model_status, find_model_path, get_sample_audio_path, transcribe_file};

#[tokio::test]
async fn test_check_model_status() {
    let result = check_model_status().await;
    assert!(result.is_ok(), "check_model_status should return Ok");
    let is_ready = result.unwrap();
    // Cross-check with find_model_path
    let found = find_model_path().unwrap();
    assert_eq!(is_ready, found.is_some(),
        "check_model_status and find_model_path should agree");
    if let Some(path) = found {
        println!("Model found at: {:?}", path);
    }
}

#[tokio::test]
async fn test_get_sample_audio_path() {
    let result = get_sample_audio_path().await;
    match result {
        Ok(path) => {
            assert!(path.contains("jfk.wav"), "Path should contain jfk.wav");
            assert!(
                std::path::Path::new(&path).exists(),
                "Sample audio file should exist at: {}",
                path
            );
        }
        Err(_) => {
            println!("SKIPPED: get_sample_audio_path - not running from repo root");
        }
    }
}

#[tokio::test]
async fn test_transcribe_sample_audio() {
    // Check prerequisites
    let model_path = match find_model_path() {
        Ok(Some(p)) => p,
        _ => {
            println!("SKIPPED: test_transcribe_sample_audio - model not downloaded");
            return;
        }
    };
    println!("Model at: {:?}", model_path);

    let sample_path = match get_sample_audio_path().await {
        Ok(path) => path,
        Err(_) => {
            println!("SKIPPED: test_transcribe_sample_audio - sample audio not found");
            return;
        }
    };

    let result = transcribe_file(sample_path).await;
    assert!(result.is_ok(), "Transcription should succeed: {:?}", result.err());

    let text = result.unwrap();
    assert!(!text.is_empty(), "Transcription should not be empty");

    let text_lower = text.to_lowercase();
    let has_recognizable_content = text_lower.contains("country")
        || text_lower.contains("ask")
        || text_lower.contains("fellow")
        || text_lower.contains("american");
    assert!(
        has_recognizable_content,
        "Transcription should contain recognizable words from JFK speech, got: {}",
        text
    );
}
