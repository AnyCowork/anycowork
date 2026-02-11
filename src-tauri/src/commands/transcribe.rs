use tauri::{command, Emitter};
use transcribe_rs::{TranscriptionEngine, engines::parakeet::ParakeetEngine};
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use serde_json::json;
use futures::StreamExt;

// Constants for model
const PARAKEET_MODEL_URL: &str = "https://blob.handy.computer/parakeet-v3-int8.tar.gz";
const MODEL_DIR_NAME: &str = "models";

fn get_models_dir() -> Result<PathBuf, String> {
    let mut path = dirs::data_local_dir().ok_or("Could not find data local dir")?;
    path.push("anycowork");
    path.push(MODEL_DIR_NAME);
    Ok(path)
}

/// Find the actual model directory by scanning for the required vocab.txt file
/// inside subdirectories of the models dir. The tar.gz may extract to any folder name.
pub fn find_model_path() -> Result<Option<PathBuf>, String> {
    let models_dir = get_models_dir()?;
    if !models_dir.exists() {
        return Ok(None);
    }

    let entries = fs::read_dir(&models_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && has_model_files(&path) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

/// Check if a directory contains the required Parakeet model files.
fn has_model_files(dir: &PathBuf) -> bool {
    let has_vocab = dir.join("vocab.txt").exists();
    let has_encoder = dir.join("encoder-model.int8.onnx").exists()
        || dir.join("encoder-model.onnx").exists();
    has_vocab && has_encoder
}

#[command]
pub async fn check_model_status() -> Result<bool, String> {
    Ok(find_model_path()?.is_some())
}

#[command]
pub async fn download_model(window: tauri::WebviewWindow) -> Result<String, String> {
    // Skip download if model already exists
    if find_model_path()?.is_some() {
        log::info!("Model already downloaded, skipping");
        return Ok("Model already available".to_string());
    }

    let models_dir = get_models_dir()?;
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    }

    log::info!("Downloading model from {}", PARAKEET_MODEL_URL);

    // Stream download with progress
    let client = reqwest::Client::new();
    let response = client
        .get(PARAKEET_MODEL_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to download model: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let tar_path = models_dir.join("parakeet_model.tar.gz");
    let mut file = fs::File::create(&tar_path)
        .map_err(|e| format!("Failed to create model file: {}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download interrupted: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;
        downloaded += chunk.len() as u64;

        let _ = window.emit(
            "model-download-progress",
            json!({
                "downloaded": downloaded,
                "total": total_size,
                "phase": "downloading"
            }),
        );
    }

    // Extract
    log::info!("Extracting model to {:?}", models_dir);
    let _ = window.emit(
        "model-download-progress",
        json!({
            "downloaded": downloaded,
            "total": total_size,
            "phase": "extracting"
        }),
    );

    let tar_gz = fs::File::open(&tar_path).map_err(|e| e.to_string())?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    archive
        .unpack(&models_dir)
        .map_err(|e| format!("Failed to unpack model: {}", e))?;

    // Cleanup tar file
    let _ = fs::remove_file(tar_path);

    // Verify extraction succeeded
    let model_path = find_model_path()
        .map_err(|e| format!("Post-extraction check failed: {}", e))?
        .ok_or("Model extraction failed: no model files found after unpacking")?;

    log::info!("Model ready at {:?}", model_path);

    let _ = window.emit(
        "model-download-progress",
        json!({
            "downloaded": downloaded,
            "total": total_size,
            "phase": "done"
        }),
    );

    Ok("Model downloaded and extracted successfully".to_string())
}

#[command]
pub async fn get_sample_audio_path() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("Failed to get CWD: {}", e))?;
    let sample_path = cwd.join("thirdparty/transcribe-rs/samples/jfk.wav");
    if sample_path.exists() {
        Ok(sample_path.to_string_lossy().to_string())
    } else {
        Err(format!(
            "Sample audio not found at: {}",
            sample_path.display()
        ))
    }
}

#[command]
pub async fn transcribe_file(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    let model_path = find_model_path()?
        .ok_or("Model not found. Please download the model first.")?;

    let mut engine = ParakeetEngine::new();
    engine.load_model(&model_path).map_err(|e| e.to_string())?;

    let result = engine
        .transcribe_file(&path_buf, None)
        .map_err(|e| e.to_string())?;
    Ok(result.text)
}
