use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const CURRENT_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub schema_version: u32,
    pub last_preset_id: String,
    pub favorites: Vec<String>,
    pub input_device: String,
    pub output_device: String,
    pub monitor_device: String,
    pub buffer_size: u32,
    pub voice_changer_enabled: bool,
    pub hear_myself_enabled: bool,
    pub is_muted: bool,
    pub noise_reduction_pct: f32,
    pub start_minimized: bool,
    pub start_with_windows: bool,
    pub reduce_motion: bool,
    pub first_run_complete: bool,
    pub hotkeys: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_CONFIG_VERSION,
            last_preset_id: "deep_voice".to_string(),
            favorites: vec!["deep_voice".to_string(), "natural".to_string(), "alien".to_string()],
            input_device: String::new(),
            output_device: String::new(),
            monitor_device: String::new(),
            buffer_size: 256,
            voice_changer_enabled: true,
            hear_myself_enabled: false,
            is_muted: false,
            noise_reduction_pct: 65.0,
            start_minimized: false,
            start_with_windows: false,
            reduce_motion: false,
            first_run_complete: false,
            hotkeys: HashMap::new(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VoiceForge");
        let _ = fs::create_dir_all(&dir);
        Self {
            config_path: dir.join("config.json"),
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Load config safely with corrupt-file backup and version migration
    pub fn load(&self) -> AppConfig {
        if !self.config_path.exists() {
            let default_cfg = AppConfig::default();
            let _ = self.save(&default_cfg);
            return default_cfg;
        }

        match fs::read_to_string(&self.config_path) {
            Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                Ok(mut cfg) => {
                    // Forward migration if version is older
                    if cfg.schema_version < CURRENT_CONFIG_VERSION {
                        cfg.schema_version = CURRENT_CONFIG_VERSION;
                        let _ = self.save(&cfg);
                    }
                    cfg
                }
                Err(err) => {
                    eprintln!("Corrupt config detected ({}). Creating backup.", err);
                    self.backup_bad_file(&content);
                    let fallback = AppConfig::default();
                    let _ = self.save(&fallback);
                    fallback
                }
            },
            Err(e) => {
                eprintln!("Failed to read config file ({}). Loading defaults.", e);
                AppConfig::default()
            }
        }
    }

    /// Atomic write: saves to temporary file then renames to prevent partial corruption
    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let json_str = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let temp_path = self.config_path.with_extension("tmp");
        {
            let mut f = File::create(&temp_path)
                .map_err(|e| format!("Could not create temp config: {}", e))?;
            f.write_all(json_str.as_bytes())
                .map_err(|e| format!("Write failed: {}", e))?;
            f.flush()
                .map_err(|e| format!("Flush failed: {}", e))?;
        }

        fs::rename(&temp_path, &self.config_path)
            .map_err(|e| format!("Failed to commit config atomically: {}", e))?;

        Ok(())
    }

    fn backup_bad_file(&self, bad_content: &str) {
        let bad_path = self.config_path.with_extension("json.bad");
        let _ = fs::write(&bad_path, bad_content);
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization_and_defaults() {
        let cfg = AppConfig::default();
        let serialized = serde_json::to_string(&cfg).expect("Serialize ok");
        let parsed: AppConfig = serde_json::from_str(&serialized).expect("Deserialize ok");
        assert_eq!(parsed.schema_version, CURRENT_CONFIG_VERSION);
        assert_eq!(parsed.buffer_size, 256);
        assert!(parsed.voice_changer_enabled);
    }

    #[test]
    fn test_corrupt_config_recovery() {
        let temp_dir = std::env::temp_dir().join("vf_config_test");
        let _ = fs::create_dir_all(&temp_dir);
        let cfg_path = temp_dir.join("config.json");

        // Write corrupt JSON
        fs::write(&cfg_path, "{ malformed json ::: ").expect("Write bad json");

        let manager = ConfigManager {
            config_path: cfg_path.clone(),
        };

        let loaded = manager.load();
        assert_eq!(loaded.schema_version, CURRENT_CONFIG_VERSION);
        assert!(temp_dir.join("config.json.bad").exists());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}
