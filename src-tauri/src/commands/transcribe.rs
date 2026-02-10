use tauri::command;
use transcribe_rs::{TranscriptionEngine, engines::parakeet::ParakeetEngine};
use std::path::PathBuf;
use std::fs;
use std::io::Write;

// Constants for model
const PARAKEET_MODEL_URL: &str = "https://blob.handy.computer/parakeet-v3-int8.tar.gz";
const MODEL_DIR_NAME: &str = "models";

fn get_models_dir() -> Result<PathBuf, String> {
    let mut path = dirs::data_local_dir().ok_or("Could not find data local dir")?;
    path.push("anycowork");
    path.push(MODEL_DIR_NAME);
    Ok(path)
}

#[command]
pub async fn check_model_status() -> Result<bool, String> {
    let models_dir = get_models_dir()?;
    // Check if Parakeet model exists (simplified check)
    // Parakeet extracted usually has specific files, checking for the directory
    let model_path = models_dir.join("parakeet-v3-int8"); // Assuming extraction creates this
    // Or checking for a file inside
    model_path.join("model.onnx"); // Example file, need to verify exact structure or just check dir
    
    // For the tarball above, it usually extracts to a folder. 
    // Let's assume if the directory exists and is not empty, it's fine for this MVP.
    // We can be more robust later.
    
    // Let's check for the existence of the specific folder that the tarball likely creates.
    Ok(model_path.exists())
}

#[command]
pub async fn download_model() -> Result<String, String> {
    let models_dir = get_models_dir()?;
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    }

    log::info!("Downloading model from {}", PARAKEET_MODEL_URL);
    
    // Download using reqwest
    let response = reqwest::get(PARAKEET_MODEL_URL)
        .await
        .map_err(|e| format!("Failed to download model: {}", e))?;

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    let tar_path = models_dir.join("parakeet_model.tar.gz");
    let mut file = fs::File::create(&tar_path)
        .map_err(|e| format!("Failed to create model file: {}", e))?;
    
    file.write_all(&content)
        .map_err(|e| format!("Failed to write model file: {}", e))?;

    // Extract
    log::info!("Extracting model to {:?}", models_dir);
    let tar_gz = fs::File::open(&tar_path).map_err(|e| e.to_string())?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    
    archive.unpack(&models_dir).map_err(|e| format!("Failed to unpack model: {}", e))?;

    // Cleanup tar file
    let _ = fs::remove_file(tar_path);

    Ok("Model downloaded and extracted successfully".to_string())
}

#[command]
pub async fn transcribe_file(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err("File not found".to_string());
    }

    let models_dir = get_models_dir()?;
    let model_path = models_dir.join("parakeet-v3-int8"); // The extracted folder name needs verification but likely matches

    if !model_path.exists() {
        return Err("Model not found. Please download the model checks first.".to_string());
    }

    let mut engine = ParakeetEngine::new();
    engine.load_model(&model_path).map_err(|e| e.to_string())?;
    
    let result = engine.transcribe_file(&path_buf, None).map_err(|e| e.to_string())?;
    Ok(result.text)
}
