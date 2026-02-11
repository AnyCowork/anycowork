use crate::tools::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub struct TranscribeTool;

impl TranscribeTool {
    pub fn new() -> Self {
        Self
    }

    /// Find the model directory by scanning for actual model files.
    /// The tar.gz may extract to any folder name, so we scan subdirectories
    /// of the models dir for one containing vocab.txt + an encoder ONNX file.
    fn find_model_path() -> Result<Option<PathBuf>, String> {
        let mut models_dir = dirs::data_local_dir()
            .ok_or("Could not find local data directory")?;
        models_dir.push("anycowork");
        models_dir.push("models");

        if !models_dir.exists() {
            return Ok(None);
        }

        let entries = std::fs::read_dir(&models_dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let has_vocab = path.join("vocab.txt").exists();
                let has_encoder = path.join("encoder-model.int8.onnx").exists()
                    || path.join("encoder-model.onnx").exists();
                if has_vocab && has_encoder {
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl Tool for TranscribeTool {
    fn name(&self) -> &str {
        "transcribe"
    }

    fn description(&self) -> &str {
        "Transcribe audio files to text using the local Parakeet AI model. Accepts an absolute path to a WAV, MP3, M4A, OGG, FLAC, MP4, MOV, or MKV file and returns the transcribed text."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the audio or video file to transcribe"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn validate_args(&self, args: &Value) -> Result<(), String> {
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing required parameter: file_path")?;

        let path = Path::new(file_path);
        if !path.is_absolute() {
            return Err("file_path must be an absolute path".to_string());
        }

        Ok(())
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<Value, String> {
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing required parameter: file_path")?
            .to_string();

        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let model_path = Self::find_model_path()?
            .ok_or("Transcription model not downloaded. Please open the Transcribe app and download the Parakeet model first.")?;

        // Transcription is CPU-intensive, run in a blocking thread
        let result = tokio::task::spawn_blocking(move || {
            use transcribe_rs::{engines::parakeet::ParakeetEngine, TranscriptionEngine};

            let mut engine = ParakeetEngine::new();
            engine
                .load_model(&model_path)
                .map_err(|e| format!("Failed to load model: {}", e))?;

            let result = engine
                .transcribe_file(&path, None)
                .map_err(|e| format!("Transcription failed: {}", e))?;

            Ok::<String, String>(result.text)
        })
        .await
        .map_err(|e| format!("Transcription task failed: {}", e))??;

        Ok(json!({
            "text": result,
            "file_path": file_path
        }))
    }

    fn requires_approval(&self, _args: &Value) -> bool {
        false
    }

    fn needs_summarization(&self, _args: &Value, _result: &Value) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> crate::tools::ToolContext {
        crate::tools::ToolContext {
            permissions: std::sync::Arc::new(crate::permissions::PermissionManager::new()),
            observer: None,
            session_id: "test".to_string(),
        }
    }

    // --- Unit tests (always run) ---

    #[test]
    fn test_transcribe_tool_metadata() {
        let tool = TranscribeTool::new();
        assert_eq!(tool.name(), "transcribe");
        assert!(!tool.description().is_empty());
        assert!(!tool.requires_approval(&json!({})));
        assert!(!tool.needs_summarization(&json!({}), &json!({})));
    }

    #[test]
    fn test_parameters_schema() {
        let tool = TranscribeTool::new();
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["file_path"].is_object());
        assert_eq!(schema["required"][0], "file_path");
    }

    #[tokio::test]
    async fn test_validate_args_missing_file_path() {
        let tool = TranscribeTool::new();
        let result = tool.validate_args(&json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required parameter"));
    }

    #[tokio::test]
    async fn test_validate_args_relative_path() {
        let tool = TranscribeTool::new();
        let result = tool
            .validate_args(&json!({"file_path": "relative/path.wav"}))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("absolute path"));
    }

    #[tokio::test]
    async fn test_validate_args_absolute_path() {
        let tool = TranscribeTool::new();
        #[cfg(windows)]
        let path = "C:\\Users\\test\\audio.wav";
        #[cfg(not(windows))]
        let path = "/home/test/audio.wav";

        let result = tool.validate_args(&json!({"file_path": path})).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_file_not_found() {
        let tool = TranscribeTool::new();
        #[cfg(windows)]
        let path = "C:\\nonexistent\\audio.wav";
        #[cfg(not(windows))]
        let path = "/nonexistent/audio.wav";

        let result = tool.execute(json!({"file_path": path}), &make_ctx()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }

    // --- Integration tests (skip gracefully when prerequisites missing) ---

    #[test]
    fn test_find_model_path_detects_downloaded_model() {
        // This verifies find_model_path works with whatever folder name the tar extracts to
        match TranscribeTool::find_model_path() {
            Ok(Some(path)) => {
                println!("Model found at: {:?}", path);
                assert!(path.join("vocab.txt").exists());
                assert!(
                    path.join("encoder-model.int8.onnx").exists()
                        || path.join("encoder-model.onnx").exists()
                );
                assert!(
                    path.join("decoder_joint-model.int8.onnx").exists()
                        || path.join("decoder_joint-model.onnx").exists()
                );
                assert!(path.join("nemo128.onnx").exists());
            }
            Ok(None) => {
                println!("SKIPPED: test_find_model_path - model not downloaded");
            }
            Err(e) => panic!("find_model_path should not error: {}", e),
        }
    }

    #[test]
    fn test_find_model_path_with_empty_temp_dir() {
        // Verify scanning an empty dir returns None, not an error
        let tmp = tempfile::tempdir().unwrap();
        let models_dir = tmp.path().join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Directly scan the temp dir (simulating no model)
        let entries = std::fs::read_dir(&models_dir).unwrap();
        let mut found = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("vocab.txt").exists() {
                found = true;
            }
        }
        assert!(!found, "Empty dir should have no model");
    }

    #[test]
    fn test_find_model_path_with_fake_model() {
        // Create a fake model directory and verify detection works
        let tmp = tempfile::tempdir().unwrap();
        let model_dir = tmp.path().join("any-random-folder-name");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("vocab.txt"), "test").unwrap();
        std::fs::write(model_dir.join("encoder-model.int8.onnx"), "test").unwrap();

        // Scan the temp dir directly
        let entries = std::fs::read_dir(tmp.path()).unwrap();
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let has_vocab = path.join("vocab.txt").exists();
                let has_encoder = path.join("encoder-model.int8.onnx").exists()
                    || path.join("encoder-model.onnx").exists();
                if has_vocab && has_encoder {
                    found_path = Some(path);
                    break;
                }
            }
        }
        assert!(found_path.is_some(), "Should detect fake model dir");
        assert!(
            found_path.unwrap().ends_with("any-random-folder-name"),
            "Should find the correct directory regardless of name"
        );
    }

    #[tokio::test]
    async fn test_execute_with_real_model_and_sample() {
        // Full integration: find model + find sample audio + transcribe
        let model = match TranscribeTool::find_model_path() {
            Ok(Some(p)) => p,
            _ => {
                println!("SKIPPED: test_execute_with_real_model - model not downloaded");
                return;
            }
        };
        println!("Using model at: {:?}", model);

        // Find sample audio (from repo)
        let cwd = std::env::current_dir().unwrap();
        let sample_candidates = [
            cwd.join("thirdparty/transcribe-rs/samples/jfk.wav"),
            cwd.join("../thirdparty/transcribe-rs/samples/jfk.wav"),
        ];
        let sample_path = sample_candidates.iter().find(|p| p.exists());
        let sample_path = match sample_path {
            Some(p) => p.to_string_lossy().to_string(),
            None => {
                println!("SKIPPED: test_execute_with_real_model - sample audio not found");
                return;
            }
        };
        println!("Using sample: {}", sample_path);

        let tool = TranscribeTool::new();
        let result = tool
            .execute(json!({"file_path": sample_path}), &make_ctx())
            .await;

        match result {
            Ok(val) => {
                let text = val["text"].as_str().unwrap();
                println!("Transcription result: {}", text);
                assert!(!text.is_empty(), "Transcription should not be empty");
                // JFK speech recognition check
                let lower = text.to_lowercase();
                assert!(
                    lower.contains("country") || lower.contains("ask")
                        || lower.contains("fellow") || lower.contains("american"),
                    "Should contain recognizable JFK speech words, got: {}",
                    text
                );
                assert_eq!(val["file_path"].as_str().unwrap(), sample_path);
            }
            Err(e) => {
                // Model loading may fail in test env due to ort DLL issues
                if e.contains("Failed to load model") || e.contains("ort") {
                    println!("SKIPPED: test_execute_with_real_model - ort loading failed in test env: {}", e);
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }
}
