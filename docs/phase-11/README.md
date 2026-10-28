# Phase 11 — Voice and lip sync

## Status: partially delivered; end-to-end spoken voice is an explicit blocker.

This phase builds the voice-pipeline parts that are real and verifiable offline and reports the native-audio parts as honest `unavailable` capabilities rather than simulating them.

## Delivered (real, tested)

- **Lip-sync viseme timeline**: a deterministic text → viseme mapping over a 15-shape mouth vocabulary, producing contiguous `{ viseme, startMs, durationMs }` frames with consecutive identical shapes merged. Needs no audio, so it is always available and fully tested.
- **Voice session state machine**: push-to-talk (`StartListening` / `StopListening`), interruption (barge-in → idle), and cancellation, as a pure tested transition function. Listening is refused when capture is unavailable.
- **Owner engine configuration**: persisted `voice-config.json` holding optional Whisper model and Piper voice paths; the status truthfully reports whether each asset is configured and present on disk.
- **Truthful capability reporting**: microphone capture, audio playback, STT, and TTS-audio are reported as `unavailable` with specific reasons; lip-sync is reported available.
- Owner "Voice" workspace tab: visible mic status, capability grid, disabled push-to-talk with the blocker explained, engine-path configuration, and a lip-sync preview that renders the computed viseme strip.
- Typed IPC contracts, runtime guards, and command-name stability for all five commands.

## Completion gate — not fully met (reported blocker)

The gate is "end-to-end voice with visible mic status and cancellation."

- **Visible mic status**: met — the view shows microphone/playback/STT/TTS status truthfully.
- **Cancellation**: met for the session state machine (cancel/interrupt → idle).
- **End-to-end spoken voice**: **not met.** Microphone capture, speech-to-text transcription, and audio playback are not implemented in this build. They are native-audio integrations (an audio-capture crate plus OS microphone permission, a whisper.cpp sidecar + GGML model, a Piper sidecar + voice model + audio output) that are not present and cannot be honestly verified here. Per the repository contract, these are surfaced as explicit `unavailable` states with blockers rather than faked.

The typed boundary (engine config, session state machine, capability reporting, viseme timeline) is in place so a future phase can wire the capture/STT/TTS sidecars behind it without contract changes.

## Blockers

1. **No microphone capture** — needs an audio-capture integration and Windows microphone permission.
2. **No STT engine** — needs a whisper.cpp sidecar and a local GGML model; the owner can configure the model path now, but transcription is not wired.
3. **No TTS audio** — needs a Piper (or compatible) sidecar, a voice model, and audio playback; the lip-sync timeline is produced regardless.

## Boundaries (not in this phase)

- No bundled speech models or binaries; the owner provisions them (as with Ollama/OpenAI engines).
- The viseme mapping is a deterministic grapheme heuristic, not phoneme-accurate forced alignment; when real TTS exists, the timeline can be scaled to measured audio duration.
- No policy executor; voice actions are direct owner actions.
