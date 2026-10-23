use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::presets::VoicePreset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetFileEnvelope {
    pub schema_version: u32,
    pub app: String,
    pub preset: VoicePreset,
}

pub struct PresetStorage {
    dir: PathBuf,
}

impl Default for PresetStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl PresetStorage {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new() -> Self {
        let base_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VoiceForge")
            .join("presets");

        let _ = fs::create_dir_all(&base_dir);
        Self { dir: base_dir }
    }

    pub fn presets_dir(&self) -> &Path {
        &self.dir
    }

    /// Save preset to disk atomically using a temporary file and rename
    pub fn save_preset(&self, preset: &VoicePreset) -> Result<PathBuf, String> {
        let envelope = PresetFileEnvelope {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            app: "VoiceForge".to_string(),
            preset: preset.clone(),
        };

        let toml_str = toml::to_string_pretty(&envelope)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let safe_filename = format!("{}.vfpreset.toml", sanitize_filename(&preset.name));
        let target_path = self.dir.join(&safe_filename);
        let temp_path = self.dir.join(format!("{}.tmp", safe_filename));

        // Atomic write
        let mut f = File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
        f.write_all(toml_str.as_bytes()).map_err(|e| format!("Failed to write file: {}", e))?;
        f.flush().map_err(|e| format!("Failed to flush file: {}", e))?;
        drop(f);

        fs::rename(&temp_path, &target_path)
            .map_err(|e| format!("Failed to commit preset atomically: {}", e))?;

        Ok(target_path)
    }

    /// Load all user presets from disk
    pub fn load_all_user_presets(&self) -> Vec<VoicePreset> {
        let mut results = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml")
                    && let Ok(preset) = Self::load_preset_from_file(&path) {
                        results.push(preset);
                }
            }
        }
        results
    }

    /// Load and validate single preset from file path
    pub fn load_preset_from_file(path: &Path) -> Result<VoicePreset, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("File read error: {}", e))?;
        
        // Try envelope format
        if let Ok(envelope) = toml::from_str::<PresetFileEnvelope>(&content) {
            if envelope.schema_version > Self::CURRENT_SCHEMA_VERSION {
                return Err(format!("Preset schema version {} is newer than supported ({})", envelope.schema_version, Self::CURRENT_SCHEMA_VERSION));
            }
            return Ok(envelope.preset);
        }

        // Direct preset fallback
        let direct: VoicePreset = toml::from_str(&content)
            .map_err(|e| format!("Invalid preset format: {}", e))?;
        Ok(direct)
    }

    /// Export preset to target path
    pub fn export_preset(preset: &VoicePreset, dest_path: &Path) -> Result<(), String> {
        let envelope = PresetFileEnvelope {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            app: "VoiceForge".to_string(),
            preset: preset.clone(),
        };
        let toml_str = toml::to_string_pretty(&envelope)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(dest_path, toml_str).map_err(|e| format!("Failed to export preset: {}", e))?;
        Ok(())
    }

    /// Import preset from external file
    pub fn import_preset(&self, source_path: &Path) -> Result<VoicePreset, String> {
        let mut preset = Self::load_preset_from_file(source_path)?;
        // Assign new ID to prevent colliding with existing preset
        preset.id = format!("imported_{}_{}", preset.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));
        self.save_preset(&preset)?;
        Ok(preset)
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_serialization_roundtrip() {
        let preset = crate::presets::create_dark_voice();
        let envelope = PresetFileEnvelope {
            schema_version: 1,
            app: "VoiceForge".to_string(),
            preset: preset.clone(),
        };

        let serialized = toml::to_string_pretty(&envelope).expect("Serialize ok");
        let parsed: PresetFileEnvelope = toml::from_str(&serialized).expect("Deserialize ok");

        assert_eq!(parsed.preset.id, preset.id);
        assert_eq!(parsed.preset.name, preset.name);
        assert_eq!(parsed.preset.effects.len(), preset.effects.len());
    }
}
