//! Voice engine for ZarishNote.
//!
//! Provides voice recording, WAV-based transcription via whisper.cpp,
//! audio device enumeration, and natural-language command parsing.
//!
//! ## Architecture
//!
//! * **Recording** – [`cpal`] captures audio from the default input device.
//!   Samples are buffered in memory; on stop they are written to a temporary
//!   WAV file using [`hound`] and then transcribed.
//! * **Transcription** – [`whisper_rs`] (a Rust wrapper around whisper.cpp)
//!   loads a GGML model and transcribes 16 kHz mono audio.
//! * **Command parsing** – the same keyword-spotting logic from the original
//!   stub (save / search / open / dictate).
//!
//! ## Threading notes
//!
//! [`cpal::Stream`] is **not** `Send` on Linux (it contains a
//! `PhantomData<*mut ()>` sentinel), so the active-recording state is held
//! in a [`thread_local!`] rather than in a `static Mutex`.  Because all
//! synchronous Tauri commands execute on the main thread this is safe.
//!
//! ## Model location
//!
//! The whisper model is looked up (in order):
//! 1. The `model_path` parameter passed to a command.
//! 2. The `ZARISHNOTE_WHISPER_MODEL` environment variable.
//! 3. `~/.zarishnote/voice/models/ggml-{tiny,base,small,medium,large}.bin`
//! 4. `/usr/share/zarishnote/voice/models/ggml-base.bin`

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::types::{AppState, VoiceDevice};

// ---------------------------------------------------------------------------
// Public return types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceRecognitionResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub command_type: String,
    pub parameters: std::collections::HashMap<String, String>,
    pub confidence: f32,
}

// ---------------------------------------------------------------------------
// Internal recording state
// ---------------------------------------------------------------------------

/// Holds resources for an in-progress recording session.
///
/// `_stream` is dropped first (field order) when the struct is dropped,
/// which stops the audio callback and lets us safely drain the buffer
/// afterwards.
struct ActiveRecording {
    /// Path of the temporary WAV file that will be created on stop.
    file_path: PathBuf,
    /// F32 sample buffer shared with the cpal callback.
    buffer: Arc<Mutex<Vec<f32>>>,
    /// Flag the callback checks; set to `false` *before* the stream is
    /// dropped to guarantee no more writes into the buffer.
    is_active: Arc<AtomicBool>,
    /// The cpal input stream – **must** be kept alive.
    _stream: cpal::Stream,
    /// Sample rate reported by the device (Hz).
    sample_rate: u32,
    /// Number of channels reported by the device.
    channels: u16,
}

// Recording state is thread-local because `cpal::Stream` is `!Send`.
thread_local! {
    static RECORDING: RefCell<Option<ActiveRecording>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// Helper: whisper model discovery
// ---------------------------------------------------------------------------

fn default_model_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".zarishnote/voice/models")
}

/// Locate a whisper GGML model file.
///
/// Resolution order:
/// 1. `custom_path` if provided and exists.
/// 2. `ZARISHNOTE_WHISPER_MODEL` env var.
/// 3. Well-known paths under `~/.zarishnote/voice/models/`.
/// 4. System-wide `/usr/share/zarishnote/voice/models/ggml-base.bin`.
fn find_whisper_model(custom_path: Option<&str>) -> Result<PathBuf, String> {
    // 1 — explicit parameter
    if let Some(p) = custom_path {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Ok(pb);
        }
        return Err(format!("Whisper model not found at custom path '{}'", p));
    }

    // 2 — environment variable
    if let Ok(env_path) = std::env::var("ZARISHNOTE_WHISPER_MODEL") {
        let pb = PathBuf::from(&env_path);
        if pb.is_file() {
            tracing::info!(
                "Using whisper model from ZARISHNOTE_WHISPER_MODEL: {}",
                env_path
            );
            return Ok(pb);
        }
        tracing::warn!(
            "ZARISHNOTE_WHISPER_MODEL set but file not found: {}",
            env_path
        );
    }

    // 3 — well-known user paths
    let base = default_model_dir();
    let filenames = [
        "ggml-base.bin",
        "ggml-small.bin",
        "ggml-medium.bin",
        "ggml-large.bin",
        "ggml-tiny.bin",
        "ggml-base.en.bin",
        "ggml-small.en.bin",
        "ggml-medium.en.bin",
    ];
    for name in &filenames {
        let pb = base.join(name);
        if pb.is_file() {
            tracing::info!("Found whisper model at {}", pb.display());
            return Ok(pb);
        }
    }

    // 4 — system path
    let sys = PathBuf::from("/usr/share/zarishnote/voice/models/ggml-base.bin");
    if sys.is_file() {
        return Ok(sys);
    }

    Err(format!(
        "Whisper model not found. \
         Download a model (e.g. ggml-base.bin) from \
         https://huggingface.co/ggerganov/whisper.cpp/tree/main \
         and place it in {} , or set the ZARISHNOTE_WHISPER_MODEL \
         environment variable.",
        default_model_dir().display()
    ))
}

// ---------------------------------------------------------------------------
// Helper: audio resampling (linear)
// ---------------------------------------------------------------------------

/// Simple linear interpolation resampler.
///
/// Used when the captured audio sample rate differs from the 16 kHz that
/// whisper.cpp expects.
fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let left = src_idx.floor() as usize;
        let right = (left + 1).min(samples.len().saturating_sub(1));
        let frac = src_idx - left as f64;

        let sample = if left < samples.len() {
            samples[left] as f64 * (1.0 - frac) + samples[right] as f64 * frac
        } else {
            0.0
        };
        output.push(sample as f32);
    }

    output
}

// ---------------------------------------------------------------------------
// Helper: WAV → f32 mono → whisper transcription
// ---------------------------------------------------------------------------

/// Read a WAV file, convert to 16 kHz mono `f32`, and run whisper
/// transcription.
fn transcribe_wav_file(
    wav_path: &std::path::Path,
    model_path: Option<&str>,
) -> Result<VoiceRecognitionResult, String> {
    // ---- open WAV ----
    let mut reader =
        hound::WavReader::open(wav_path).map_err(|e| format!("Failed to open WAV: {}", e))?;

    let spec = reader.spec();
    let src_rate = spec.sample_rate;
    let channels = spec.channels as usize;

    tracing::info!(
        "WAV file: {} Hz, {} ch, {} bit {:?}",
        src_rate,
        channels,
        spec.bits_per_sample,
        spec.sample_format
    );

    // ---- read & convert to f32 ----
    let raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            match spec.bits_per_sample {
                16 => reader
                    .samples::<i16>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / max)
                    .collect(),
                32 => reader
                    .samples::<i32>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / max)
                    .collect(),
                8 => reader
                    .samples::<i8>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / max)
                    .collect(),
                b => return Err(format!("Unsupported bits per sample: {}", b)),
            }
        }
        hound::SampleFormat::Float => reader.samples::<f32>().filter_map(Result::ok).collect(),
    };

    // ---- collapse to mono ----
    let mono: Vec<f32> = if channels > 1 {
        raw.chunks(channels)
            .map(|ch| ch.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        raw
    };

    // ---- resample to 16 kHz (whisper requirement) ----
    let audio = if src_rate != 16000 {
        tracing::info!("Resampling from {} Hz to 16000 Hz", src_rate);
        resample_audio(&mono, src_rate, 16000)
    } else {
        mono
    };

    if audio.is_empty() {
        return Ok(VoiceRecognitionResult {
            text: String::new(),
            confidence: 0.0,
            language: "en-US".into(),
        });
    }

    // ---- locate model ----
    let model_path = find_whisper_model(model_path)?;

    // ---- transcribe ----
    tracing::info!(
        "Loading whisper model from {} ({} samples)",
        model_path.display(),
        audio.len()
    );

    let ctx = whisper_rs::WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        whisper_rs::WhisperContextParameters::default(),
    )
    .map_err(|e| format!("Failed to load whisper model: {}", e))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create whisper state: {}", e))?;

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) as std::ffi::c_int;

    let mut params =
        whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(n_threads);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_timestamps(false);
    params.set_no_timestamps(true);
    params.set_suppress_non_speech_tokens(true);

    state
        .full(params, &audio)
        .map_err(|e| format!("Whisper transcription failed: {}", e))?;

    let n_segments = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;

    let mut text = String::with_capacity((n_segments as usize) * 64);
    for i in 0..n_segments {
        if let Ok(seg) = state.full_get_segment_text(i) {
            text.push_str(&seg);
            text.push(' ');
        }
    }

    let text = text.trim().to_string();

    // -- language detection --
    let language = match state.full_lang_id_from_state() {
        Ok(lang_id) if lang_id >= 0 => whisper_rs::get_lang_str(lang_id)
            .unwrap_or("en")
            .to_string(),
        _ => "en".to_string(),
    };

    tracing::info!(
        "Transcription complete: {} chars, {} segments, language={}",
        text.len(),
        n_segments,
        language
    );

    Ok(VoiceRecognitionResult {
        text,
        confidence: 0.85, // whisper-rs does not expose a per-utterance confidence
        language,
    })
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Start recording audio from the default input device.
///
/// Returns the path of the temporary WAV file that will be written when
/// [`voice_stop_recording`] is called.
#[tauri::command]
pub fn voice_start_recording(_state: State<'_, AppState>) -> Result<String, String> {
    // Guard: only one recording at a time
    let already = RECORDING.with(|r| r.borrow().is_some());
    if already {
        return Err("Already recording. Stop the current recording first.".into());
    }

    // ---- resolve input device ----
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No audio input device available".to_string())?;

    let device_name = device.name().unwrap_or_else(|_| "<unknown>".into());
    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default input config: {}", e))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    tracing::info!(
        "Input device: {}  |  {} Hz  |  {} ch  |  {:?}",
        device_name,
        sample_rate,
        channels,
        config.sample_format()
    );

    // ---- temp file ----
    let file_path =
        std::env::temp_dir().join(format!("zarishnote_rec_{}.wav", uuid::Uuid::new_v4()));

    // ---- shared state for the callback ----
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(4096)));
    let buffer_clone = Arc::clone(&buffer);

    let is_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let is_active_clone = Arc::clone(&is_active);

    let err_fn = move |err| {
        tracing::error!("Audio stream error: {}", err);
    };

    // ---- build input stream ----
    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if is_active_clone.load(Ordering::SeqCst) {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        buf.extend_from_slice(data);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Failed to build audio stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start stream: {}", e))?;

    // ---- persist state (thread-local) ----
    let recording = ActiveRecording {
        file_path: file_path.clone(),
        buffer,
        is_active,
        _stream: stream,
        sample_rate,
        channels,
    };

    RECORDING.with(|r| {
        *r.borrow_mut() = Some(recording);
    });

    tracing::info!("Recording started → {}", file_path.display());

    Ok(file_path.to_string_lossy().into_owned())
}

/// Check whether a recording is currently in progress.
#[tauri::command]
pub fn voice_is_recording() -> Result<bool, String> {
    Ok(RECORDING.with(|r| r.borrow().is_some()))
}

/// Stop the active recording, write the temporary WAV file, transcribe it
/// with whisper, and return the recognised text.
#[tauri::command]
pub fn voice_stop_recording() -> Result<VoiceRecognitionResult, String> {
    let active = RECORDING
        .with(|r| r.borrow_mut().take())
        .ok_or_else(|| "No active recording to stop".to_string())?;

    // 1. prevent the callback from writing any more samples
    active.is_active.store(false, Ordering::SeqCst);

    // 2. drop the stream – this stops the background audio thread.
    //    (fields are dropped in declaration order; _stream is declared
    //     after buffer and is_active, so it is dropped first in reverse
    //     declaration order … actually Rust drops in declaration order,
    //     so let's be explicit.)
    let ActiveRecording {
        file_path,
        buffer,
        is_active: _,
        _stream,
        sample_rate,
        channels,
    } = active;

    drop(_stream); // stop audio capture

    // 3. drain the sample buffer
    let samples: Vec<f32> = {
        let mut buf = buffer.lock().map_err(|e| e.to_string())?;
        std::mem::take(&mut *buf)
    };

    tracing::info!(
        "Recording stopped — {} samples ({} ch, {} Hz)",
        samples.len(),
        channels,
        sample_rate
    );

    if samples.is_empty() {
        let _ = std::fs::remove_file(&file_path);
        return Ok(VoiceRecognitionResult {
            text: String::new(),
            confidence: 0.0,
            language: "en-US".into(),
        });
    }

    // 4. collapse to mono
    let mono: Vec<f32> = if channels > 1 {
        samples
            .chunks(channels as usize)
            .map(|ch| ch.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };

    // 5. resample to 16 kHz for whisper
    let audio_16k = if sample_rate != 16000 {
        resample_audio(&mono, sample_rate, 16000)
    } else {
        mono
    };

    // 6. write 16-bit mono 16 kHz WAV
    {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(&file_path, spec)
            .map_err(|e| format!("Failed to create WAV: {}", e))?;

        for &s in &audio_16k {
            let clamped = s.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    }

    tracing::info!("WAV written to {}", file_path.display());

    // 7. transcribe
    let result = transcribe_wav_file(&file_path, None)?;

    // 8. clean up temp file
    if file_path.exists() {
        let _ = std::fs::remove_file(&file_path);
    }

    Ok(result)
}

/// Transcribe an existing audio file.
///
/// * `path` – path to a WAV file on disk.
/// * `model_path` – optional path to a whisper GGML model; uses the default
///   search strategy otherwise.
#[tauri::command]
pub fn voice_transcribe_file(
    path: String,
    model_path: Option<String>,
) -> Result<VoiceRecognitionResult, String> {
    let wav_path = PathBuf::from(&path);
    if !wav_path.is_file() {
        return Err(format!("Audio file not found: {}", path));
    }

    tracing::info!("Transcribing file: {}", path);
    transcribe_wav_file(&wav_path, model_path.as_deref())
}

/// List all available audio input devices.
#[tauri::command]
pub fn voice_list_devices() -> Result<Vec<VoiceDevice>, String> {
    let host = cpal::default_host();

    let default_device = host.default_input_device();
    let default_name: Option<String> = default_device.as_ref().and_then(|d| d.name().ok());

    let devices: Vec<VoiceDevice> = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate input devices: {}", e))?
        .filter_map(|device| {
            let name = device.name().ok()?;
            let config = device.default_input_config().ok()?;
            let is_default = default_name.as_deref() == Some(name.as_str());
            let id = if is_default {
                "default".to_string()
            } else {
                name.clone()
            };

            Some(VoiceDevice {
                id,
                name,
                is_default,
                channels: config.channels(),
            })
        })
        .collect();

    tracing::info!("Found {} audio input devices", devices.len());

    Ok(devices)
}

/// Get metadata about an audio file (duration, sample rate, etc.).
#[tauri::command]
pub fn voice_get_audio_info(path: String) -> Result<crate::types::AudioFileInfo, String> {
    let wav_path = PathBuf::from(&path);
    if !wav_path.is_file() {
        return Err(format!("Audio file not found: {}", path));
    }

    let reader =
        hound::WavReader::open(&wav_path).map_err(|e| format!("Failed to open WAV: {}", e))?;

    let spec = reader.spec();
    let total_samples = reader.len() as f64;
    let duration_secs = total_samples / spec.sample_rate as f64 / spec.channels as f64;

    Ok(crate::types::AudioFileInfo {
        path,
        duration_secs,
        sample_rate: spec.sample_rate,
    })
}

// ---------------------------------------------------------------------------
// Command parsing (preserved from the original stub)
// ---------------------------------------------------------------------------

/// Parse transcribed text for known voice commands.
///
/// Recognised keywords:
/// * **save** – save the current note.
/// * **search <query>** – initiate a search.
/// * **open <path>** – open a note by name.
/// * Everything else is treated as **dictate** (plain text insertion).
#[tauri::command]
pub fn voice_process_command(
    _state: State<'_, AppState>,
    text: String,
) -> Result<VoiceCommand, String> {
    let lower = text.to_lowercase();

    let (command_type, parameters) = if lower.contains("save") {
        ("save".into(), std::collections::HashMap::new())
    } else if lower.contains("search") {
        let query = text
            .splitn(2, "search")
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();
        let mut params = std::collections::HashMap::new();
        params.insert("query".into(), query);
        ("search".into(), params)
    } else if lower.contains("open") {
        let file = text
            .splitn(2, "open")
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();
        let mut params = std::collections::HashMap::new();
        params.insert("file".into(), file);
        ("open".into(), params)
    } else {
        let mut params = std::collections::HashMap::new();
        params.insert("text".into(), text);
        ("dictate".into(), params)
    };

    Ok(VoiceCommand {
        command_type,
        parameters,
        confidence: 0.9,
    })
}
