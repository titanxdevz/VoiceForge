pub mod comms;
pub mod device;
pub mod mmcss;
pub mod soundboard;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, Stream, StreamConfig};
use crossbeam_channel::{Receiver, Sender};
use rtrb::RingBuffer;

use crate::audio::comms::{AudioCommand, AudioTelemetry};
use crate::audio::device::DeviceManager;
use crate::audio::mmcss::set_audio_thread_priority;
use crate::dsp::chain::EffectChain;
use crate::presets::VoicePreset;

pub struct AudioEngine {
    device_manager: DeviceManager,
    is_running: Arc<AtomicBool>,
    dropout_counter: Arc<AtomicU64>,
    cmd_sender: Sender<AudioCommand>,
    cmd_receiver: Receiver<AudioCommand>,
    // Telemetry channel (Audio thread -> UI thread)
    telem_sender: Sender<AudioTelemetry>,
    telem_receiver: Receiver<AudioTelemetry>,

    // Active CPAL streams
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    monitor_stream: Option<Stream>,

    // Current device selections
    pub current_input_device: String,
    pub current_output_device: String,
    pub current_monitor_device: String,
    pub current_sample_rate: u32,
    pub current_buffer_size: u32,
    pub monitoring_enabled: bool,
    pub monitoring_active: Arc<AtomicBool>,
}

impl AudioEngine {
    pub const DEFAULT_BUFFER_SIZE: u32 = 256;
    pub const RINGBUFFER_CAPACITY: usize = 8192;

    pub fn new() -> Result<Self, String> {
        let dm = DeviceManager::new()?;
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (telem_tx, telem_rx) = crossbeam_channel::unbounded();

        let in_dev = dm.list_input_devices().into_iter().find(|d| d.is_default).map(|d| d.name).unwrap_or_default();
        let out_dev = dm.list_output_devices().into_iter().find(|d| d.is_vbcable).map(|d| d.name)
            .or_else(|| dm.list_output_devices().into_iter().find(|d| d.is_default).map(|d| d.name))
            .unwrap_or_default();

        let mon_dev = dm.list_output_devices().into_iter().find(|d| !d.is_vbcable && d.is_default).map(|d| d.name)
            .or_else(|| dm.list_output_devices().into_iter().find(|d| !d.is_vbcable).map(|d| d.name))
            .unwrap_or_default();

        Ok(Self {
            device_manager: dm,
            is_running: Arc::new(AtomicBool::new(false)),
            dropout_counter: Arc::new(AtomicU64::new(0)),
            cmd_sender: cmd_tx,
            cmd_receiver: cmd_rx,
            telem_sender: telem_tx,
            telem_receiver: telem_rx,
            input_stream: None,
            output_stream: None,
            monitor_stream: None,
            current_input_device: in_dev,
            current_output_device: out_dev,
            current_monitor_device: mon_dev,
            current_sample_rate: 48000,
            current_buffer_size: Self::DEFAULT_BUFFER_SIZE,
            monitoring_enabled: false,
            monitoring_active: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn device_manager(&self) -> &DeviceManager {
        &self.device_manager
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn set_monitoring(&mut self, enabled: bool) {
        self.monitoring_enabled = enabled;
        self.monitoring_active.store(enabled, Ordering::Relaxed);
        self.send_command(AudioCommand::SetMonitoring(enabled));
    }

    pub fn send_command(&self, cmd: AudioCommand) {
        let _ = self.cmd_sender.send(cmd);
    }

    pub fn try_receive_telemetry(&self) -> Option<AudioTelemetry> {
        let mut last = None;
        while let Ok(t) = self.telem_receiver.try_recv() {
            last = Some(t);
        }
        last
    }

    pub fn start(&mut self, initial_preset: VoicePreset) -> Result<(), String> {
        if self.is_running() {
            self.stop();
        }

        let in_device = self.device_manager.find_input_device(&self.current_input_device)
            .ok_or_else(|| "Could not find input audio device".to_string())?;
        let out_device = self.device_manager.find_output_device(&self.current_output_device)
            .ok_or_else(|| "Could not find output audio device".to_string())?;

        // Query device supported default configurations to ensure 100% WASAPI compatibility
        let in_supported = in_device.default_input_config()
            .map_err(|e| format!("Default input config error: {}", e))?;
        let in_channels = in_supported.channels() as usize;
        let mut in_config: StreamConfig = in_supported.into();
        in_config.buffer_size = BufferSize::Default;

        let out_supported = out_device.default_output_config()
            .map_err(|e| format!("Default output config error: {}", e))?;
        let out_channels = out_supported.channels() as usize;
        let mut out_config: StreamConfig = out_supported.into();
        out_config.buffer_size = BufferSize::Default;

        self.current_sample_rate = in_config.sample_rate;

        // Ring buffer between Input capture callback and DSP processing
        let (mut in_prod, mut in_cons) = RingBuffer::<f32>::new(Self::RINGBUFFER_CAPACITY);
        // Ring buffer between DSP processing and Output playback callback
        let (mut out_prod, mut out_cons) = RingBuffer::<f32>::new(Self::RINGBUFFER_CAPACITY);

        let is_running = self.is_running.clone();
        let dropouts = self.dropout_counter.clone();
        let cmd_rx = self.cmd_receiver.clone();
        let telem_tx = self.telem_sender.clone();
        let mon_active = self.monitoring_active.clone();
        let is_out_vbcable = DeviceManager::is_cable_device(&self.current_output_device);

        // Optional monitor stream for headphones when output is VB-CABLE
        let mut maybe_mon_prod = None;
        let mut built_monitor_stream = None;

        if is_out_vbcable {
            let mon_dev_opt = self.device_manager.find_output_device(&self.current_monitor_device)
                .or_else(|| {
                    self.device_manager.list_output_devices().into_iter()
                        .find(|d| !d.is_vbcable)
                        .and_then(|d| self.device_manager.find_output_device(&d.name))
                });

            if let Some(mon_dev) = mon_dev_opt
                && let Ok(mon_supp) = mon_dev.default_output_config() {
                    let mon_channels = mon_supp.channels() as usize;
                    let mut mon_cfg: StreamConfig = mon_supp.into();
                    mon_cfg.buffer_size = BufferSize::Default;

                    let (m_prod, mut m_cons) = RingBuffer::<f32>::new(Self::RINGBUFFER_CAPACITY);
                    let mon_active_cb = mon_active.clone();

                    if let Ok(m_stream) = mon_dev.build_output_stream(
                        mon_cfg,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            let allow = mon_active_cb.load(Ordering::Relaxed);
                            for frame in data.chunks_mut(mon_channels) {
                                let sample = m_cons.pop().unwrap_or(0.0);
                                let val = if allow { sample } else { 0.0 };
                                for ch in frame.iter_mut() {
                                    *ch = val;
                                }
                            }
                        },
                        |err| eprintln!("Monitor stream error: {:?}", err),
                        None,
                    ) {
                        let _ = m_stream.play();
                        built_monitor_stream = Some(m_stream);
                        maybe_mon_prod = Some(m_prod);
                    }
                }
        }

        // 1. Build input stream (supports mono, stereo, or multi-mic array)
        let in_dropouts = dropouts.clone();
        let input_stream = in_device.build_input_stream(
            in_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if in_channels == 1 {
                    for &sample in data {
                        if in_prod.push(sample).is_err() {
                            in_dropouts.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                } else {
                    for chunk in data.chunks(in_channels) {
                        let sum: f32 = chunk.iter().sum();
                        let mono = sum / (in_channels as f32);
                        if in_prod.push(mono).is_err() {
                            in_dropouts.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Input audio stream error: {:?}", err);
            },
            None,
        ).map_err(|e| format!("Failed to build input stream: {}", e))?;

        // 2. Build output stream
        // CRITICAL BUG FIX: If output is physical Speakers/Headphones, do NOT play sound unless "Hear Myself" is ON!
        // If output is VB-CABLE, always play sound to VB-CABLE for Discord/Games.
        let out_dropouts = dropouts.clone();
        let mon_active_out = mon_active.clone();
        let output_stream = out_device.build_output_stream(
            out_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let allow_output = is_out_vbcable || mon_active_out.load(Ordering::Relaxed);

                for frame in data.chunks_mut(out_channels) {
                    let sample = out_cons.pop().unwrap_or_else(|_| {
                        out_dropouts.fetch_add(1, Ordering::Relaxed);
                        0.0
                    });
                    let out_val = if allow_output { sample } else { 0.0 };
                    for ch in frame.iter_mut() {
                        *ch = out_val;
                    }
                }
            },
            move |err| {
                eprintln!("Output audio stream error: {:?}", err);
            },
            None,
        ).map_err(|e| format!("Failed to build output stream: {}", e))?;

        // 3. Spawn dedicated high-priority DSP thread
        let dsp_running = is_running.clone();
        let dsp_sample_rate = self.current_sample_rate as f32;
        let dsp_mon_active = mon_active.clone();

        std::thread::Builder::new()
            .name("VoiceForge-DSP".to_string())
            .spawn(move || {
                set_audio_thread_priority();

                let mut chain = EffectChain::new(dsp_sample_rate);
                apply_preset_to_chain(&mut chain, &initial_preset);

                let mut soundboard_buffer: Vec<f32> = Vec::new();
                let mut soundboard_index = 0usize;

                let block_size = 128;
                let mut dsp_block = vec![0.0f32; block_size];
                let mut telem_timer = Instant::now();

                let mut in_peak = 0.0f32;
                let mut out_peak = 0.0f32;
                let mut in_sum_sq = 0.0f32;
                let mut out_sum_sq = 0.0f32;
                let mut processed_samples = 0;

                while dsp_running.load(Ordering::Relaxed) {
                    // Check commands
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            AudioCommand::SetPreset(p) => apply_preset_to_chain(&mut chain, &p),
                            AudioCommand::SetBypass(b) => chain.is_bypassed = b,
                            AudioCommand::SetMasterGain(g) => chain.set_master_gain_db(g),
                            AudioCommand::SetMonitoring(b) => {
                                dsp_mon_active.store(b, Ordering::Relaxed);
                            }
                            AudioCommand::SetPitchFormant { pitch, formant } => {
                                for eff in chain.effects.iter_mut() {
                                    if let crate::dsp::chain::EffectInstance::PitchFormant(pf) = &mut eff.instance {
                                        let mut params = pf.params().clone();
                                        params.pitch_semitones = pitch;
                                        params.formant_shift_pct = formant;
                                        pf.set_params(params);
                                    }
                                }
                            }
                            AudioCommand::PlaySoundboardSample(s) => {
                                soundboard_buffer = s;
                                soundboard_index = 0;
                            }
                            AudioCommand::StopSoundboard => {
                                soundboard_buffer.clear();
                                soundboard_index = 0;
                            }
                            _ => {}
                        }
                    }

                    // Read block from input ring buffer
                    if in_cons.slots() >= block_size {
                        let proc_start = Instant::now();
                        for slot in dsp_block.iter_mut().take(block_size) {
                            let s = in_cons.pop().unwrap_or(0.0);
                            *slot = s;
                            let abs = s.abs();
                            if abs > in_peak { in_peak = abs; }
                            in_sum_sq += s * s;
                        }

                        // Mix soundboard if active
                        if soundboard_index < soundboard_buffer.len() {
                            for s in dsp_block.iter_mut() {
                                if soundboard_index < soundboard_buffer.len() {
                                    *s = (*s * 0.7) + (soundboard_buffer[soundboard_index] * 0.8);
                                    soundboard_index += 1;
                                }
                            }
                        }

                        // Run effect chain
                        chain.process_block(&mut dsp_block);

                        // Push block to output ring buffer and monitor ring buffer
                        let is_mon = dsp_mon_active.load(Ordering::Relaxed);
                        for &s in &dsp_block {
                            let _ = out_prod.push(s);
                            if let Some(ref mut mp) = maybe_mon_prod
                                && is_mon {
                                    let _ = mp.push(s);
                            }
                            let abs = s.abs();
                            if abs > out_peak { out_peak = abs; }
                            out_sum_sq += s * s;
                        }

                        let proc_time = proc_start.elapsed().as_secs_f32();
                        let block_duration = block_size as f32 / dsp_sample_rate;
                        let cpu_pct = (proc_time / block_duration) * 100.0;
                        processed_samples += block_size;

                        // Telemetry update at ~30Hz
                        if telem_timer.elapsed().as_millis() >= 33 {
                            let in_rms = (in_sum_sq / processed_samples as f32).sqrt();
                            let out_rms = (out_sum_sq / processed_samples as f32).sqrt();

                            let in_db = if in_peak > 1e-5 { 20.0 * in_peak.log10() } else { -90.0 };
                            let out_db = if out_peak > 1e-5 { 20.0 * out_peak.log10() } else { -90.0 };
                            let in_rms_db = if in_rms > 1e-5 { 20.0 * in_rms.log10() } else { -90.0 };
                            let out_rms_db = if out_rms > 1e-5 { 20.0 * out_rms.log10() } else { -90.0 };

                            let latency_ms = ((block_size * 2) as f32 / dsp_sample_rate) * 1000.0;

                            let _ = telem_tx.send(AudioTelemetry {
                                input_peak_db: in_db,
                                output_peak_db: out_db,
                                input_rms_db: in_rms_db,
                                output_rms_db: out_rms_db,
                                estimated_latency_ms: latency_ms,
                                cpu_load_pct: cpu_pct.clamp(0.0, 100.0),
                                dropouts: dropouts.load(Ordering::Relaxed),
                                is_running: true,
                                sample_rate: dsp_sample_rate as u32,
                                buffer_size: block_size,
                            });

                            in_peak = 0.0;
                            out_peak = 0.0;
                            in_sum_sq = 0.0;
                            out_sum_sq = 0.0;
                            processed_samples = 0;
                            telem_timer = Instant::now();
                        }
                    } else {
                        // Spin / sleep lightly to yield
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn DSP thread: {}", e))?;

        input_stream.play().map_err(|e| format!("Input stream play error: {}", e))?;
        output_stream.play().map_err(|e| format!("Output stream play error: {}", e))?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
        self.monitor_stream = built_monitor_stream;
        self.is_running.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(stream) = self.input_stream.take() {
            let _ = stream.pause();
        }
        if let Some(stream) = self.output_stream.take() {
            let _ = stream.pause();
        }
        if let Some(stream) = self.monitor_stream.take() {
            let _ = stream.pause();
        }
    }
}

fn apply_preset_to_chain(chain: &mut EffectChain, preset: &VoicePreset) {
    chain.effects.clear();
    chain.set_master_gain_db(preset.master_gain_db);
    for eff in &preset.effects {
        chain.add_effect(eff.clone());
    }
}
