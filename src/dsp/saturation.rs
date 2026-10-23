use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SaturationMode {
    SoftClip,
    TubeWarmth,
    HardDistortion,
    AsymmetricDrive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaturatorParams {
    pub drive_db: f32, // 0.0 .. 30.0 dB
    pub mode: SaturationMode,
    pub bias: f32,     // -0.5 .. +0.5 (even harmonics generation)
    pub dry_wet: f32,  // 0.0 .. 1.0
}

impl Default for SaturatorParams {
    fn default() -> Self {
        Self {
            drive_db: 4.0,
            mode: SaturationMode::TubeWarmth,
            bias: 0.05,
            dry_wet: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Saturator {
    params: SaturatorParams,
    drive_linear: f32,
    output_trim: f32,
}

impl Saturator {
    pub fn new(params: SaturatorParams) -> Self {
        let mut sat = Self {
            params,
            drive_linear: 1.0,
            output_trim: 1.0,
        };
        sat.recalculate();
        sat
    }

    pub fn set_params(&mut self, params: SaturatorParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        self.drive_linear = 10.0_f32.powf(self.params.drive_db / 20.0);
        // Automatic gain compensation trim
        self.output_trim = 1.0 / (1.0 + (self.drive_linear - 1.0) * 0.45);
    }

    pub fn reset(&mut self) {}

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let driven = (input + self.params.bias) * self.drive_linear;

        let saturated = match self.params.mode {
            SaturationMode::SoftClip => {
                // Smooth cubic polynomial saturation
                let x = driven.clamp(-1.5, 1.5);
                x - (x * x * x) / 4.5
            }
            SaturationMode::TubeWarmth => {
                // Arctan / Tanh gentle saturation
                (driven * 0.9).tanh()
            }
            SaturationMode::HardDistortion => {
                driven.clamp(-1.0, 1.0)
            }
            SaturationMode::AsymmetricDrive => {
                // Classic guitar / villain vocal diode curve
                if driven >= 0.0 {
                    1.0 - (-driven).exp()
                } else {
                    -1.0 + (driven).exp()
                }
            }
        };

        let post = (saturated - self.params.bias) * self.output_trim;
        let wet = self.params.dry_wet.clamp(0.0, 1.0);
        let out = input * (1.0 - wet) + post * wet;

        if out.is_nan() || out.is_infinite() { 0.0 } else { out }
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}
