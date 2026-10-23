# Changelog

All notable changes to **VoiceForge** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-10-22

### Added
- **Real-Time DSP Audio Engine**:
  - Independent Pitch Shift (±12 semitones) and Formant Shift (±50%) using SOLA/WSOLA and 3-stage vocal tract formant filters.
  - Direct Form II Transposed Biquad Filters (LowPass, HighPass, BandPass, Notch, PeakingEQ, LowShelf, HighShelf).
  - Dynamics Suite: Soft-knee Compressor, Fast Downward Noise Gate with Hysteresis, Side-chain De-Esser, and Always-On Brickwall Limiter.
  - Multi-waveform Ring Modulator (Sine, Square, Sawtooth, Pulse-Train) for robotic and vocoder voices.
  - Multi-mode Non-linear Saturator (Tube Warmth, Cubic Soft-Clip, Hard Distortion, Asymmetric Drive).
  - 4-Delay-Line Feedback Delay Network (FDN) Reverb with frequency-dependent damping and stereo width control.
- **Voice Library (25 Presets)**:
  - Deep Voice, Dark Voice, Demon, Villain, Cartoon, Chipmunk, Kid, Giant, Robot, Vocoder Robot, Radio, Telephone, Megaphone, Walkie-Talkie, Alien, Ghost, Whisper, Echo, Hall, Cave, Stadium, Anonymous, Male↔Female Shift, Natural, Custom.
  - 1-line acoustic description and distinctive "how it sounds" tag per preset.
- **UI / UX (Tool Aesthetic)**:
  - Custom Ableton / Reaper dark palette (#0B0F19 Base, #141C2B Surface, #2563EB Accent, #60A5FA Secondary).
  - Hairline 1px borders with zero neon gradient slop.
  - Custom vector rotary knobs with value indicators and drag sensitivity.
  - Vertical dB meters with peak-hold ballistics.
  - Slim navigation: Dashboard, Voice Studio, Voice Library, Audio & Devices, Performance & Diagnostics.
- **Hardware & Windows Integration**:
  - WASAPI low-latency audio device enumeration via CPAL.
  - Automatic VB-CABLE virtual audio device detection and feedback loop safety guard.
  - Windows MMCSS "Pro Audio" thread scheduler registration (`AvSetMmThreadCharacteristicsW`).
  - System tray icon with context menu (Start/Stop, Mute, Open, Quit).
  - Single-instance enforcement via Windows Named Mutex.
  - Current-user registry autostart toggle (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
  - Lock-free `rtrb` RingBuffer pipelines ensuring zero allocations on the audio callback.
- **Presets & Storage**:
  - Atomic TOML preset persistence with schema versioning.
  - Import, export, duplicate, rename, and favorite toggles.
- **Soundboard**:
  - Multi-format audio file loader (WAV, MP3, OGG, FLAC) mixed into the real-time stream.
- **Diagnostics**:
  - Real-time latency meter, dropout counter, and DSP CPU load factor.
  - Non-audio JSON diagnostic telemetry export.
