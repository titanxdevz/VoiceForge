use std::time::Instant;

// Benchmark audio block throughput
#[test]
fn bench_dsp_chain_throughput() {
    let mut chain = VoiceForge::dsp::chain::EffectChain::new(48000.0);
    
    // Configure full chain: Gate -> Pitch/Formant -> EQ -> Saturation -> Reverb -> Limiter
    chain.add_effect(VoiceForge::dsp::chain::ChainEffectConfig {
        effect_type: VoiceForge::dsp::chain::EffectType::NoiseGate,
        name: "Gate".to_string(),
        enabled: true,
        wet_dry: 1.0,
        params: VoiceForge::dsp::chain::EffectParams::NoiseGate(Default::default()),
    });
    chain.add_effect(VoiceForge::dsp::chain::ChainEffectConfig {
        effect_type: VoiceForge::dsp::chain::EffectType::PitchFormant,
        name: "Pitch & Formant".to_string(),
        enabled: true,
        wet_dry: 1.0,
        params: VoiceForge::dsp::chain::EffectParams::PitchFormant(VoiceForge::dsp::pitch_formant::PitchFormantParams {
            pitch_semitones: -4.0,
            formant_shift_pct: -25.0,
            dry_wet: 1.0,
        }),
    });
    chain.add_effect(VoiceForge::dsp::chain::ChainEffectConfig {
        effect_type: VoiceForge::dsp::chain::EffectType::Equalizer,
        name: "EQ".to_string(),
        enabled: true,
        wet_dry: 1.0,
        params: VoiceForge::dsp::chain::EffectParams::Equalizer(Default::default()),
    });
    chain.add_effect(VoiceForge::dsp::chain::ChainEffectConfig {
        effect_type: VoiceForge::dsp::chain::EffectType::Saturation,
        name: "Saturation".to_string(),
        enabled: true,
        wet_dry: 0.5,
        params: VoiceForge::dsp::chain::EffectParams::Saturation(Default::default()),
    });
    chain.add_effect(VoiceForge::dsp::chain::ChainEffectConfig {
        effect_type: VoiceForge::dsp::chain::EffectType::Reverb,
        name: "Reverb".to_string(),
        enabled: true,
        wet_dry: 0.4,
        params: VoiceForge::dsp::chain::EffectParams::Reverb(Default::default()),
    });

    let block_size = 256;
    let mut block = vec![0.1f32; block_size];

    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        chain.process_block(&mut block);
    }
    let elapsed = start.elapsed();
    let total_audio_duration_sec = (iterations * block_size) as f64 / 48000.0;
    let processing_time_sec = elapsed.as_secs_f64();
    let realtime_ratio = processing_time_sec / total_audio_duration_sec;

    println!(
        "\n=== DSP Benchmark Results ===\nTotal audio simulated: {:.3} s\nTime taken to compute: {:.5} s\nReal-time load factor: {:.2}% of single core\nPer-block DSP latency: {:.3} microseconds\n============================\n",
        total_audio_duration_sec,
        processing_time_sec,
        realtime_ratio * 100.0,
        (elapsed.as_micros() as f64) / (iterations as f64)
    );

    assert!(realtime_ratio < 0.25, "DSP load must be well under 25% for real-time safety, was {:.2}%", realtime_ratio * 100.0);
}
