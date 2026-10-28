import { useEffect, useState } from "react";
import type { LipSync, VoiceStatus } from "@saiki/ipc-contracts";
import { cancelVoice, configureVoice, getVoiceStatus, interruptVoice, previewLipSync, startListening, stopListening } from "./ipc";
import "./voice.css";

export function VoiceView() {
  const [voice, setVoice] = useState<VoiceStatus>();
  const [message, setMessage] = useState("Reading voice capabilities…");
  const [sttPath, setSttPath] = useState("");
  const [ttsPath, setTtsPath] = useState("");
  const [text, setText] = useState("");
  const [lipSync, setLipSync] = useState<LipSync>();

  useEffect(() => { void getVoiceStatus().then(apply).catch(fail); }, []);

  function fail(error: unknown): void { setMessage(error instanceof Error ? error.message : String(error)); }
  function apply(status: VoiceStatus): void { setVoice(status); setSttPath(status.sttModel.path ?? ""); setTtsPath(status.ttsVoice.path ?? ""); setMessage(status.listeningAvailable ? "Voice capture is available." : "Spoken voice is unavailable in this build; lip-sync works."); }

  function saveConfig(): void { void configureVoice({ sttModelPath: sttPath.trim() || null, ttsVoicePath: ttsPath.trim() || null }).then((status) => { apply(status); setMessage("Engine paths saved."); }).catch(fail); }
  function pushToTalk(): void { void startListening().then(apply).catch((error: unknown) => { setMessage(error instanceof Error ? error.message : String(error)); }); }
  function stop(): void { void stopListening().then(apply).catch(fail); }
  function interrupt(): void { void interruptVoice().then(apply).catch(fail); }
  function cancel(): void { void cancelVoice().then(apply).catch(fail); }
  function preview(): void { const value = text.trim(); if (!value) return; void previewLipSync(value).then((result) => { setLipSync(result); setMessage(`Lip-sync timeline: ${result.visemes.length} frame${result.visemes.length === 1 ? "" : "s"}, ~${result.estimatedDurationMs} ms.`); }).catch(fail); }

  if (!voice) return <div className="view-stack"><p className="provider-message" role="status">{message}</p></div>;

  return <div className="view-stack voice">
    <div className="section-heading"><div><p className="eyebrow">VOICE · LIP-SYNC</p><h1>Voice</h1></div><span className={`mic-badge ${voice.microphone.available ? "ok" : "off"}`}>MIC: {voice.microphone.available ? "READY" : "UNAVAILABLE"}</span></div>
    <p className="provider-message" role="status">{message}</p>

    <div className="capability-grid">
      <Capability label="Microphone capture" ok={voice.microphone.available} note={voice.microphone.reason} />
      <Capability label="Audio playback" ok={voice.playback.available} note={voice.playback.reason} />
      <Capability label="Whisper STT model" ok={voice.sttModel.present} note={voice.sttModel.configured ? (voice.sttModel.present ? "Configured and found on disk" : "Configured but file not found") : "Not configured"} />
      <Capability label="Piper TTS voice" ok={voice.ttsVoice.present} note={voice.ttsVoice.configured ? (voice.ttsVoice.present ? "Configured and found on disk" : "Configured but file not found") : "Not configured"} />
      <Capability label="Lip-sync timeline" ok={voice.lipSyncAvailable} note="Deterministic, local, no audio required" />
    </div>

    <div className="ptt-row"><button className="ptt" onClick={pushToTalk} disabled={!voice.listeningAvailable}>Push to talk</button><button className="quiet-button" onClick={stop} disabled={voice.session !== "listening"}>Stop</button><button className="quiet-button" onClick={interrupt} disabled={voice.session !== "speaking"}>Interrupt</button><button className="quiet-button" onClick={cancel}>Cancel</button><span className="session">session: {voice.session}</span></div>
    {!voice.listeningAvailable && <p className="scope-note">Listening is disabled: speech-to-text needs a Whisper model and microphone capture, which is not wired in this build. Configure an engine path below; capture and playback remain a reported blocker until implemented.</p>}

    <div className="engine-config">
      <h2>Speech engine assets</h2>
      <label className="model-field"><span>Whisper model path (.gguf/.bin)</span><input value={sttPath} maxLength={1024} placeholder="C:\\models\\ggml-base.en.bin" onChange={(event) => setSttPath(event.target.value)} /></label>
      <label className="model-field"><span>Piper voice path (.onnx)</span><input value={ttsPath} maxLength={1024} placeholder="C:\\voices\\en_US-amy.onnx" onChange={(event) => setTtsPath(event.target.value)} /></label>
      <div className="model-actions"><button onClick={saveConfig}>Save engine paths</button></div>
    </div>

    <div className="lipsync-form">
      <h2>Lip-sync preview</h2>
      <label className="model-field"><span>Text for Saiki to mouth</span><textarea value={text} maxLength={8192} rows={2} onChange={(event) => setText(event.target.value)} /></label>
      <div className="model-actions"><button onClick={preview} disabled={!text.trim()}>Preview lip-sync</button></div>
      {lipSync && <div className="viseme-strip"><div className="viseme-reason">Audio: {lipSync.audioOutcome} — {lipSync.audioReason}</div><div className="frames">{lipSync.visemes.map((frame, index) => <span key={index} className="frame" style={{ flexGrow: frame.durationMs }} title={`${frame.startMs}–${frame.startMs + frame.durationMs} ms`}>{frame.viseme}</span>)}</div></div>}
    </div>
  </div>;
}

function Capability({ label, ok, note }: { label: string; ok: boolean; note: string }) {
  return <article className={`capability ${ok ? "ok" : "off"}`}><div className="dot" /><div><b>{label}</b><small>{note}</small></div></article>;
}
