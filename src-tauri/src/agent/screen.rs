//! Hotkey screen-ask. Captures pixels in RAM, then a local vision model
//! answers. This is not [`super::computer::TAKE_SCREENSHOT`], which saves a file.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, RgbaImage};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use super::config::AgentConfig;
use super::error::{AgentError, AgentResult};
use super::ids;
use super::runtime::AgentRuntime;
use super::store;

pub const EVENT: &str = "coda://screen-ask";
pub const DEFAULT_HOTKEY: &str = "CommandOrControl+Shift+C";
pub const DEFAULT_QUESTION: &str = "What's on my screen?";

const MAX_EDGE: u32 = 1280;
const JPEG_QUALITY: u8 = 70;
const VISION_TIMEOUT: Duration = Duration::from_secs(180);

static IN_FLIGHT: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenAskEvent {
    pub phase: &'static str,
    pub question: String,
    pub text: String,
    pub error: Option<String>,
}

pub fn register_hotkey(app: &tauri::App) {
    let handle = app.handle().clone();
    handle.state::<AgentRuntime>().ensure_loaded(&handle);
    let hotkey = handle.state::<AgentRuntime>().screen_hotkey();
    let shortcut = if hotkey.trim().is_empty() {
        DEFAULT_HOTKEY.to_string()
    } else {
        hotkey
    };
    match app.global_shortcut().on_shortcut(shortcut.as_str(), move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let Some(runtime) = app.try_state::<AgentRuntime>() else {
                return;
            };
            if let Err(err) = runtime.ask_screen(&app, None).await {
                log::warn!("screen-ask hotkey: {err}");
            }
        });
    }) {
        Ok(()) => log::info!("screen-ask hotkey registered ({shortcut})"),
        Err(err) => log::warn!("could not register screen-ask hotkey {shortcut}: {err}"),
    }
}

pub async fn ask_screen(
    app: &AppHandle,
    http: &reqwest::Client,
    config: &AgentConfig,
    question: &str,
) -> AgentResult<String> {
    if IN_FLIGHT
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AgentError::msg("Already looking at the screen."));
    }
    let result = ask_screen_inner(app, http, config, question).await;
    IN_FLIGHT.store(false, Ordering::SeqCst);
    result
}

async fn ask_screen_inner(
    app: &AppHandle,
    http: &reqwest::Client,
    config: &AgentConfig,
    question: &str,
) -> AgentResult<String> {
    let question = {
        let trimmed = question.trim();
        if trimmed.is_empty() {
            DEFAULT_QUESTION.to_string()
        } else {
            trimmed.to_string()
        }
    };

    let captured = match std::thread::spawn(capture_frame)
        .join()
        .unwrap_or_else(|_| Err(AgentError::msg("Screen capture failed.")))
    {
        Ok(frame) => frame,
        Err(err) => {
            reveal_window(app);
            emit_event(app, "error", &question, "", Some(err.to_string()));
            return Err(err);
        }
    };

    let mut saved_path = None;
    if config.save_screen_captures {
        saved_path = save_capture(&captured.jpeg)?;
    }

    reveal_window(app);
    emit_event(app, "streaming", &question, "", None);

    let host = config.ollama_url.trim().trim_end_matches('/');
    let names = list_models(http, host).await?;
    let Some(model) = pick_vision_model(&names, config.vision_model.as_deref()) else {
        let err = AgentError::msg(
            "Vision model is not installed. Run `ollama pull moondream` (or `ollama pull llava`).",
        );
        emit_event(app, "error", &question, "", Some(err.to_string()));
        return Err(err);
    };

    let _ = store::append_audit(
        app,
        "screen.ask",
        &json!({
            "question": question,
            "model": model,
            "source": captured.source,
            "width": captured.width,
            "height": captured.height,
            "saved": saved_path,
        }),
    );

    match stream_vision(http, host, &model, &question, &captured.b64, app).await {
        Ok(text) => {
            emit_event(app, "done", &question, &text, None);
            Ok(text)
        }
        Err(err) => {
            emit_event(app, "error", &question, "", Some(err.to_string()));
            Err(err)
        }
    }
}

struct CapturedFrame {
    jpeg: Vec<u8>,
    b64: String,
    width: u32,
    height: u32,
    source: &'static str,
}

fn capture_frame() -> AgentResult<CapturedFrame> {
    let (rgba, source) = grab_rgba()?;
    if is_blank(&rgba) {
        return Err(permission_error());
    }
    let jpeg = encode_jpeg(&rgba)?;
    if jpeg.is_empty() {
        return Err(AgentError::msg("Screen image was empty."));
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(&jpeg);
    Ok(CapturedFrame {
        width: rgba.width(),
        height: rgba.height(),
        jpeg,
        b64,
        source,
    })
}

fn grab_rgba() -> AgentResult<(RgbaImage, &'static str)> {
    let windows = xcap::Window::all().map_err(|err| map_capture_error(&err.to_string()))?;
    if let Some(window) = pick_window(&windows) {
        match window.capture_image() {
            Ok(image) if !is_blank(&image) => return Ok((image, "window")),
            Ok(_) => {}
            Err(err) => {
                let text = err.to_string();
                if looks_like_permission(&text) {
                    return Err(permission_error());
                }
            }
        }
    }

    let monitors = xcap::Monitor::all().map_err(|err| map_capture_error(&err.to_string()))?;
    let monitor = monitors
        .iter()
        .find(|item| item.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or_else(permission_error)?;
    let image = monitor
        .capture_image()
        .map_err(|err| map_capture_error(&err.to_string()))?;
    if is_blank(&image) {
        return Err(permission_error());
    }
    Ok((image, "monitor"))
}

fn pick_window(windows: &[xcap::Window]) -> Option<&xcap::Window> {
    windows
        .iter()
        .find(|window| {
            !is_own_window(window)
                && window.is_focused().unwrap_or(false)
                && !window.is_minimized().unwrap_or(false)
        })
        .or_else(|| {
            windows.iter().find(|window| {
                !is_own_window(window)
                    && !window.is_minimized().unwrap_or(false)
                    && window.width().unwrap_or(0) > 32
                    && window.height().unwrap_or(0) > 32
            })
        })
}

fn is_own_window(window: &xcap::Window) -> bool {
    let app = window
        .app_name()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let title = window.title().unwrap_or_default().to_ascii_lowercase();
    app.contains("coda")
        || app.contains("ghostnote")
        || title == "coda"
        || title.contains("coda —")
}

fn is_blank(image: &RgbaImage) -> bool {
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return true;
    }
    let total = width.saturating_mul(height) as usize;
    let step = (total / 250).max(1);
    let mut dark = 0usize;
    let mut n = 0usize;
    for (index, pixel) in image.pixels().enumerate() {
        if index % step != 0 {
            continue;
        }
        n += 1;
        if pixel.0[0] < 8 && pixel.0[1] < 8 && pixel.0[2] < 8 {
            dark += 1;
        }
    }
    n > 8 && dark.saturating_mul(100) / n > 98
}

fn encode_jpeg(rgba: &RgbaImage) -> AgentResult<Vec<u8>> {
    let mut frame: RgbaImage = rgba.clone();
    let max_dim = frame.width().max(frame.height());
    if max_dim > MAX_EDGE {
        let scale = MAX_EDGE as f32 / max_dim as f32;
        let width = ((frame.width() as f32) * scale).max(1.0) as u32;
        let height = ((frame.height() as f32) * scale).max(1.0) as u32;
        frame = image::imageops::resize(&frame, width, height, image::imageops::FilterType::Triangle);
    }
    let rgb = image::DynamicImage::ImageRgba8(frame).to_rgb8();
    let mut out = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|err| AgentError::msg(format!("Could not encode the screen image: {err}")))?;
    Ok(out)
}

fn save_capture(jpeg: &[u8]) -> AgentResult<Option<String>> {
    let dir = crate::coda::agent_dir().join("screen-ask");
    std::fs::create_dir_all(&dir)
        .map_err(|err| AgentError::msg(format!("Could not save the screen capture: {err}")))?;
    let path = dir.join(format!("{}.jpg", ids::now_ms()));
    std::fs::write(&path, jpeg)
        .map_err(|err| AgentError::msg(format!("Could not save the screen capture: {err}")))?;
    Ok(Some(path.display().to_string()))
}

fn permission_error() -> AgentError {
    AgentError::msg(
        "Could not see the screen. Enable Screen Recording for Coda in System Settings → Privacy & Security, then try again.",
    )
}

fn map_capture_error(text: &str) -> AgentError {
    if looks_like_permission(text) {
        permission_error()
    } else {
        AgentError::msg(format!(
            "Could not see the screen. Enable Screen Recording for Coda. ({text})"
        ))
    }
}

fn looks_like_permission(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("permission")
        || lower.contains("denied")
        || lower.contains("not authorized")
        || lower.contains("screen recording")
        || lower.contains("screencapturekit")
}

fn reveal_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn emit_event(app: &AppHandle, phase: &'static str, question: &str, text: &str, error: Option<String>) {
    let _ = app.emit(
        EVENT,
        ScreenAskEvent {
            phase,
            question: question.to_string(),
            text: text.to_string(),
            error,
        },
    );
}

pub fn pick_vision_model(names: &[String], preferred: Option<&str>) -> Option<String> {
    let mut wants = Vec::new();
    if let Some(value) = preferred.map(str::trim).filter(|value| !value.is_empty()) {
        wants.push(value.to_string());
    }
    for name in ["moondream", "llava", "llava:7b", "llava:latest"] {
        if !wants.iter().any(|item| item.eq_ignore_ascii_case(name)) {
            wants.push(name.to_string());
        }
    }
    for want in wants {
        if let Some(hit) = names.iter().find(|name| model_matches(name, &want)) {
            return Some(hit.clone());
        }
    }
    None
}

fn model_matches(name: &str, want: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let want = want.to_ascii_lowercase();
    name == want || name.starts_with(&format!("{want}:")) || name.starts_with(&want)
}

async fn list_models(http: &reqwest::Client, host: &str) -> AgentResult<Vec<String>> {
    let url = format!("{host}/api/tags");
    let response = http
        .get(&url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|_| AgentError::msg(format!(
            "Ollama is not reachable at {host}. Set OLLAMA_HOST and run `ollama serve`."
        )))?;
    if !response.status().is_success() {
        return Err(AgentError::msg(format!(
            "Ollama is not reachable at {host} (status {}).",
            response.status()
        )));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    Ok(body
        .get("models")
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter_map(|model| model.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

async fn stream_vision(
    http: &reqwest::Client,
    host: &str,
    model: &str,
    question: &str,
    image_b64: &str,
    app: &AppHandle,
) -> AgentResult<String> {
    let url = format!("{host}/api/chat");
    let payload = json!({
        "model": model,
        "stream": true,
        "keep_alive": "60m",
        "messages": [{
            "role": "user",
            "content": question,
            "images": [image_b64]
        }]
    });
    let mut response = http
        .post(url)
        .timeout(VISION_TIMEOUT)
        .json(&payload)
        .send()
        .await
        .map_err(|_| AgentError::msg(format!(
            "Ollama is not reachable at {host}. Set OLLAMA_HOST and run `ollama serve`."
        )))?;
    if !response.status().is_success() {
        return Err(AgentError::msg(format!(
            "Vision request failed ({}) using {model}. Try `ollama pull {model}`.",
            response.status()
        )));
    }

    let mut buffer: Vec<u8> = Vec::new();
    let mut accumulated = String::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?
    {
        buffer.extend_from_slice(&chunk);
        while let Some(newline) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=newline).collect();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if let Some(piece) = value
                .get("message")
                .and_then(|message| message.get("content"))
                .and_then(Value::as_str)
            {
                if !piece.is_empty() {
                    accumulated.push_str(piece);
                    emit_event(app, "streaming", question, &accumulated, None);
                }
            }
            if value.get("error").and_then(Value::as_str).is_some() {
                let err = value["error"].as_str().unwrap_or("vision model error");
                return Err(AgentError::msg(err.to_string()));
            }
        }
    }
    let text = accumulated.trim();
    if text.is_empty() {
        return Err(AgentError::msg(
            "the vision model returned nothing usable. Pull moondream (`ollama pull moondream`).",
        ));
    }
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn prefers_moondream_then_llava() {
        let names = vec!["llama3.1:8b".into(), "moondream:latest".into(), "llava:7b".into()];
        assert_eq!(
            pick_vision_model(&names, None).as_deref(),
            Some("moondream:latest")
        );
        assert_eq!(
            pick_vision_model(&names, Some("llava")).as_deref(),
            Some("llava:7b")
        );
    }

    #[test]
    fn missing_vision_model() {
        let names = vec!["llama3.1:8b".into(), "qwen2.5:3b".into()];
        assert!(pick_vision_model(&names, None).is_none());
    }

    #[test]
    fn black_frame_is_blank() {
        let image = RgbaImage::from_pixel(32, 32, Rgba([0, 0, 0, 255]));
        assert!(is_blank(&image));
        let red = RgbaImage::from_pixel(32, 32, Rgba([200, 10, 10, 255]));
        assert!(!is_blank(&red));
    }

    #[test]
    fn jpeg_encode_is_nonempty_and_not_a_file() {
        let image = RgbaImage::from_pixel(48, 32, Rgba([12, 80, 200, 255]));
        let jpeg = encode_jpeg(&image).expect("jpeg");
        assert!(jpeg.len() > 32);
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
    }
}
