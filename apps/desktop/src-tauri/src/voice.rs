//! Voice and lip-sync, owned by the Rust core.
//!
//! This phase delivers the parts of the voice pipeline that are real and
//! verifiable offline: a deterministic text → viseme timeline for lip-sync, a
//! push-to-talk / interruption / cancellation session state machine, owner
//! configuration of speech-engine asset paths, and truthful capability
//! reporting.
//!
//! Microphone capture, speech-to-text transcription, and audio playback are
//! native-audio integrations that are **not implemented in this build**. They
//! are reported as explicit `unavailable` capabilities with blockers rather
//! than simulated. End-to-end spoken voice therefore remains unavailable until
//! those engines and a microphone are wired; the lip-sync timeline is produced
//! regardless, since it needs no audio.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::State;

const MAX_SPEAK_BYTES: usize = 8 * 1024;
const MAX_PATH_BYTES: usize = 1024;
const MIC_UNAVAILABLE: &str =
    "Microphone capture is not implemented in this build; no audio is recorded.";
const PLAYBACK_UNAVAILABLE: &str =
    "Audio playback is not implemented in this build; synthesized speech is not played.";
const STT_UNAVAILABLE: &str = "Speech-to-text requires a configured Whisper model and microphone capture, which is not wired in this build.";
const TTS_AUDIO_UNAVAILABLE: &str = "Text-to-speech audio requires a configured Piper voice and audio playback, which is not wired in this build.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Viseme {
    Silence,
    Pp,
    Ff,
    Th,
    Dd,
    Kk,
    Ch,
    Ss,
    Nn,
    Rr,
    Aa,
    Ee,
    Ih,
    Oh,
    Ou,
}

impl Viseme {
    fn duration_ms(self) -> u32 {
        match self {
            Viseme::Aa | Viseme::Ee | Viseme::Ih | Viseme::Oh | Viseme::Ou => 120,
            Viseme::Silence => 80,
            _ => 60,
        }
    }
}

fn viseme_for(ch: char) -> Option<Viseme> {
    let lower = ch.to_ascii_lowercase();
    let viseme = match lower {
        'a' => Viseme::Aa,
        'e' => Viseme::Ee,
        'i' | 'y' => Viseme::Ih,
        'o' => Viseme::Oh,
        'u' | 'w' => Viseme::Ou,
        'p' | 'b' | 'm' => Viseme::Pp,
        'f' | 'v' => Viseme::Ff,
        't' | 'd' => Viseme::Dd,
        'k' | 'g' | 'c' | 'q' => Viseme::Kk,
        'j' => Viseme::Ch,
        'n' => Viseme::Nn,
        'r' => Viseme::Rr,
        's' | 'z' | 'x' => Viseme::Ss,
        'h' | 'l' => Viseme::Th,
        _ if lower.is_whitespace() || lower.is_ascii_punctuation() => Viseme::Silence,
        _ => return None,
    };
    Some(viseme)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisemeFrame {
    viseme: Viseme,
    start_ms: u32,
    duration_ms: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LipSync {
    visemes: Vec<VisemeFrame>,
    estimated_duration_ms: u32,
    audio_outcome: &'static str,
    audio_reason: &'static str,
}

/// Build a deterministic viseme timeline for lip-sync. Consecutive identical
/// mouth shapes are merged into one longer frame. This needs no audio, so it
/// is always available; when real TTS exists later, the timeline can be scaled
/// to the synthesized audio's measured duration.
fn compute_lip_sync(text: &str) -> LipSync {
    let mut frames: Vec<VisemeFrame> = Vec::new();
    let mut cursor = 0u32;
    for ch in text.chars() {
        let Some(viseme) = viseme_for(ch) else {
            continue;
        };
        let duration = viseme.duration_ms();
        if let Some(last) = frames.last_mut() {
            if last.viseme == viseme {
                last.duration_ms += duration;
                cursor += duration;
                continue;
            }
        }
        frames.push(VisemeFrame {
            viseme,
            start_ms: cursor,
            duration_ms: duration,
        });
        cursor += duration;
    }
    LipSync {
        visemes: frames,
        estimated_duration_ms: cursor,
        audio_outcome: "unavailable",
        audio_reason: TTS_AUDIO_UNAVAILABLE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Listening,
    Speaking,
}

/// Voice session events. The transitions are real even though audio-gated
/// events cannot currently fire, because capture and playback are unavailable.
#[derive(Debug, Clone, Copy)]
pub enum VoiceEvent {
    StartListening,
    StopListening,
    Interrupt,
    Cancel,
}

#[derive(Debug, Clone, Copy)]
struct Capabilities {
    can_listen: bool,
}

fn transition(
    current: SessionState,
    event: VoiceEvent,
    caps: Capabilities,
) -> Result<SessionState, String> {
    match event {
        VoiceEvent::StartListening => {
            if !caps.can_listen {
                return Err(STT_UNAVAILABLE.into());
            }
            Ok(SessionState::Listening)
        }
        // Stop only ends an active listen; otherwise the state is unchanged.
        VoiceEvent::StopListening => Ok(if current == SessionState::Listening {
            SessionState::Idle
        } else {
            current
        }),
        // Interruption (barge-in) stops speech immediately; a no-op otherwise.
        VoiceEvent::Interrupt => Ok(if current == SessionState::Speaking {
            SessionState::Idle
        } else {
            current
        }),
        VoiceEvent::Cancel => Ok(SessionState::Idle),
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VoiceConfig {
    #[serde(default)]
    stt_model_path: Option<String>,
    #[serde(default)]
    tts_voice_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineAsset {
    configured: bool,
    present: bool,
    path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    available: bool,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStatus {
    stt_model: EngineAsset,
    tts_voice: EngineAsset,
    microphone: Capability,
    playback: Capability,
    listening_available: bool,
    speech_audio_available: bool,
    lip_sync_available: bool,
    session: SessionState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigureVoiceRequest {
    #[serde(default)]
    stt_model_path: Option<String>,
    #[serde(default)]
    tts_voice_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LipSyncRequest {
    text: String,
}

pub struct VoiceState {
    config_path: PathBuf,
    config: Mutex<VoiceConfig>,
    session: Mutex<SessionState>,
}

impl VoiceState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        let config_path = config_dir.join("voice-config.json");
        let config = match fs::read(&config_path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|_| "voice configuration is invalid")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => VoiceConfig::default(),
            Err(_) => return Err("voice configuration is unavailable".into()),
        };
        Ok(Self {
            config_path,
            config: Mutex::new(config),
            session: Mutex::new(SessionState::Idle),
        })
    }

    fn status(&self) -> Result<VoiceStatus, String> {
        let config = self.config.lock().map_err(|_| "voice state unavailable")?;
        let session = *self.session.lock().map_err(|_| "voice state unavailable")?;
        Ok(build_status(&config, session))
    }

    fn caps(&self) -> Capabilities {
        // Listening needs microphone capture, which is not wired in this build.
        Capabilities { can_listen: false }
    }
}

fn engine_asset(path: &Option<String>) -> EngineAsset {
    let present = path
        .as_deref()
        .map(|value| Path::new(value).is_file())
        .unwrap_or(false);
    EngineAsset {
        configured: path.is_some(),
        present,
        path: path.clone(),
    }
}

fn build_status(config: &VoiceConfig, session: SessionState) -> VoiceStatus {
    VoiceStatus {
        stt_model: engine_asset(&config.stt_model_path),
        tts_voice: engine_asset(&config.tts_voice_path),
        microphone: Capability {
            available: false,
            reason: MIC_UNAVAILABLE,
        },
        playback: Capability {
            available: false,
            reason: PLAYBACK_UNAVAILABLE,
        },
        listening_available: false,
        speech_audio_available: false,
        lip_sync_available: true,
        session,
    }
}

fn persist(path: &Path, config: &VoiceConfig) -> Result<(), String> {
    let parent = path.parent().ok_or("voice configuration path is invalid")?;
    fs::create_dir_all(parent).map_err(|_| "create voice configuration directory failed")?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(config).map_err(|_| "serialize voice configuration failed")?,
    )
    .map_err(|_| "write voice configuration failed")?;
    fs::rename(&temporary, path).map_err(|_| "commit voice configuration failed".to_string())
}

fn normalize_path(value: Option<String>) -> Result<Option<String>, String> {
    match value {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.len() > MAX_PATH_BYTES {
                return Err("engine path is too long".into());
            }
            Ok(Some(trimmed.to_owned()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_voice_status(state: State<'_, VoiceState>) -> Result<VoiceStatus, String> {
    state.status()
}

#[tauri::command]
pub fn configure_voice(
    state: State<'_, VoiceState>,
    request: ConfigureVoiceRequest,
) -> Result<VoiceStatus, String> {
    let stt = normalize_path(request.stt_model_path)?;
    let tts = normalize_path(request.tts_voice_path)?;
    {
        let mut config = state.config.lock().map_err(|_| "voice state unavailable")?;
        config.stt_model_path = stt;
        config.tts_voice_path = tts;
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn preview_lip_sync(request: LipSyncRequest) -> Result<LipSync, String> {
    if request.text.trim().is_empty() || request.text.len() > MAX_SPEAK_BYTES {
        return Err("text must be 1-8192 bytes".into());
    }
    Ok(compute_lip_sync(&request.text))
}

#[tauri::command]
pub fn start_listening(state: State<'_, VoiceState>) -> Result<VoiceStatus, String> {
    let caps = state.caps();
    let mut session = state
        .session
        .lock()
        .map_err(|_| "voice state unavailable")?;
    *session = transition(*session, VoiceEvent::StartListening, caps)?;
    drop(session);
    state.status()
}

#[tauri::command]
pub fn stop_listening(state: State<'_, VoiceState>) -> Result<VoiceStatus, String> {
    let caps = state.caps();
    let mut session = state
        .session
        .lock()
        .map_err(|_| "voice state unavailable")?;
    *session = transition(*session, VoiceEvent::StopListening, caps)?;
    drop(session);
    state.status()
}

#[tauri::command]
pub fn interrupt_voice(state: State<'_, VoiceState>) -> Result<VoiceStatus, String> {
    let caps = state.caps();
    let mut session = state
        .session
        .lock()
        .map_err(|_| "voice state unavailable")?;
    *session = transition(*session, VoiceEvent::Interrupt, caps)?;
    drop(session);
    state.status()
}

#[tauri::command]
pub fn cancel_voice(state: State<'_, VoiceState>) -> Result<VoiceStatus, String> {
    let caps = state.caps();
    let mut session = state
        .session
        .lock()
        .map_err(|_| "voice state unavailable")?;
    *session = transition(*session, VoiceEvent::Cancel, caps)?;
    drop(session);
    state.status()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lip_sync_is_deterministic_and_merges_repeats() {
        let a = compute_lip_sync("papa");
        let b = compute_lip_sync("papa");
        assert_eq!(a.visemes, b.visemes);
        assert_eq!(a.visemes.len(), 4);
        assert_eq!(a.visemes[0].viseme, Viseme::Pp);
        assert_eq!(a.visemes[1].viseme, Viseme::Aa);
        assert!(a.estimated_duration_ms > 0);

        // Consecutive identical visemes merge into one longer frame.
        let merged = compute_lip_sync("mmm");
        assert_eq!(merged.visemes.len(), 1);
        assert_eq!(merged.visemes[0].viseme, Viseme::Pp);
        assert_eq!(merged.visemes[0].duration_ms, 3 * Viseme::Pp.duration_ms());
    }

    #[test]
    fn lip_sync_frames_are_contiguous() {
        let lip = compute_lip_sync("hello there");
        let mut cursor = 0u32;
        for frame in &lip.visemes {
            assert_eq!(frame.start_ms, cursor);
            cursor += frame.duration_ms;
        }
        assert_eq!(cursor, lip.estimated_duration_ms);
    }

    #[test]
    fn empty_text_is_rejected_but_blank_computes_nothing() {
        assert!(compute_lip_sync("").visemes.is_empty());
        // Spaces map to silence frames, not nothing.
        assert_eq!(compute_lip_sync("  ").visemes.len(), 1);
        assert_eq!(compute_lip_sync("  ").visemes[0].viseme, Viseme::Silence);
    }

    #[test]
    fn lip_sync_reports_audio_unavailable() {
        let lip = compute_lip_sync("hi");
        assert_eq!(lip.audio_outcome, "unavailable");
        assert!(!lip.audio_reason.is_empty());
    }

    #[test]
    fn listening_is_refused_without_capture() {
        let caps = Capabilities { can_listen: false };
        assert!(transition(SessionState::Idle, VoiceEvent::StartListening, caps).is_err());
    }

    #[test]
    fn interruption_and_cancel_return_to_idle() {
        let caps = Capabilities { can_listen: true };
        assert_eq!(
            transition(SessionState::Speaking, VoiceEvent::Interrupt, caps).unwrap(),
            SessionState::Idle
        );
        assert_eq!(
            transition(SessionState::Listening, VoiceEvent::Cancel, caps).unwrap(),
            SessionState::Idle
        );
    }

    #[test]
    fn status_reports_honest_unavailable_capabilities() {
        let status = build_status(&VoiceConfig::default(), SessionState::Idle);
        assert!(status.lip_sync_available);
        assert!(!status.listening_available);
        assert!(!status.microphone.available);
        assert!(!status.playback.available);
    }

    #[test]
    fn configuration_persists_and_reflects_presence() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("whisper.gguf");
        std::fs::write(&model, b"stub").unwrap();
        {
            let state = VoiceState::load(dir.path()).unwrap();
            let mut config = state.config.lock().unwrap();
            config.stt_model_path = Some(model.to_str().unwrap().to_owned());
            persist(&state.config_path, &config).unwrap();
        }
        let reloaded = VoiceState::load(dir.path()).unwrap();
        let status = reloaded.status().unwrap();
        assert!(status.stt_model.configured);
        assert!(status.stt_model.present);
        assert!(!status.tts_voice.configured);
    }
}
