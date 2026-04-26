use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Manager};

const PIPER_MODEL_NAME: &str = "en_US-lessac-medium";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsRequest {
    input: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsResponse {
    audio_base64: String,
    cached: bool,
    engine: String,
}

fn cache_key(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(PIPER_MODEL_NAME.as_bytes());
    hasher.update(b"\n");
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn bundled_piper_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("MOMOLITE_PIPER_DIR") {
        let path = PathBuf::from(dir);
        if path.join("piper.exe").exists() {
            return Ok(path);
        }
    }

    let dev_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("piper");
    if dev_dir.join("piper.exe").exists() {
        return Ok(dev_dir);
    }

    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|error| format!("failed to resolve resource dir: {error}"))?;
    let bundled_dir = resource_dir.join("piper");
    if bundled_dir.join("piper.exe").exists() {
        return Ok(bundled_dir);
    }

    Err("Piper TTS resource not found".to_string())
}

#[tauri::command]
pub fn synthesize_piper_tts(app: AppHandle, request: TtsRequest) -> Result<TtsResponse, String> {
    let input = request.input.trim();
    if input.is_empty() {
        return Err("text is empty".to_string());
    }
    if input.len() > 2000 {
        return Err("text is too long for one TTS request".to_string());
    }

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("failed to resolve cache dir: {error}"))?
        .join("tts")
        .join("piper");
    fs::create_dir_all(&cache_dir)
        .map_err(|error| format!("failed to create TTS cache dir: {error}"))?;

    let cache_path = cache_dir.join(format!("{}.wav", cache_key(input)));
    if cache_path.exists() {
        let bytes =
            fs::read(&cache_path).map_err(|error| format!("failed to read TTS cache: {error}"))?;
        return Ok(TtsResponse {
            audio_base64: STANDARD.encode(bytes),
            cached: true,
            engine: "Piper en_US-lessac-medium".to_string(),
        });
    }

    let piper_dir = bundled_piper_dir(&app)?;
    let piper_exe = piper_dir.join("piper.exe");
    let model_path = piper_dir.join("en_US-lessac-medium.onnx");
    let config_path = piper_dir.join("en_US-lessac-medium.onnx.json");
    if !model_path.exists() || !config_path.exists() {
        return Err("Piper English voice model is missing".to_string());
    }

    let mut child = Command::new(&piper_exe)
        .arg("--model")
        .arg(&model_path)
        .arg("--config")
        .arg(&config_path)
        .arg("--output_file")
        .arg(&cache_path)
        .current_dir(&piper_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start Piper TTS: {error}"))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "failed to open Piper stdin".to_string())?;
        stdin
            .write_all(input.as_bytes())
            .map_err(|error| format!("failed to write text to Piper: {error}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for Piper TTS: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Piper TTS failed: {stderr}"));
    }

    let bytes =
        fs::read(&cache_path).map_err(|error| format!("failed to read Piper audio: {error}"))?;
    Ok(TtsResponse {
        audio_base64: STANDARD.encode(bytes),
        cached: false,
        engine: "Piper en_US-lessac-medium".to_string(),
    })
}
