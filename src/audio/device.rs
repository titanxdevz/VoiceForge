use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

#[derive(Debug, Clone, PartialEq)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub is_vbcable: bool,
    pub supported_sample_rates: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct VbCableStatus {
    pub is_installed: bool,
    pub input_device_name: Option<String>,
    pub output_device_name: Option<String>,
    pub feedback_warning: bool,
}

pub struct DeviceManager {
    host: Host,
}

impl DeviceManager {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        Ok(Self { host })
    }

    pub fn list_input_devices(&self) -> Vec<AudioDeviceInfo> {
        let default_name = self.host.default_input_device().map(|d| d.to_string());
        let mut list = Vec::new();

        if let Ok(devices) = self.host.input_devices() {
            for d in devices {
                let name = d.to_string();
                let is_default = default_name.as_deref() == Some(&name);
                let is_vbcable = Self::is_cable_device(&name);
                let rates = Self::get_supported_rates(&d);

                list.push(AudioDeviceInfo {
                    name,
                    is_default,
                    is_vbcable,
                    supported_sample_rates: rates,
                });
            }
        }
        list
    }

    pub fn list_output_devices(&self) -> Vec<AudioDeviceInfo> {
        let default_name = self.host.default_output_device().map(|d| d.to_string());
        let mut list = Vec::new();

        if let Ok(devices) = self.host.output_devices() {
            for d in devices {
                let name = d.to_string();
                let is_default = default_name.as_deref() == Some(&name);
                let is_vbcable = Self::is_cable_device(&name);
                let rates = Self::get_supported_rates(&d);

                list.push(AudioDeviceInfo {
                    name,
                    is_default,
                    is_vbcable,
                    supported_sample_rates: rates,
                });
            }
        }
        list
    }

    pub fn detect_vbcable(&self, current_input: &str, current_output: &str) -> VbCableStatus {
        let inputs = self.list_input_devices();
        let outputs = self.list_output_devices();

        let cable_input = inputs.iter().find(|d| d.is_vbcable).map(|d| d.name.clone());
        let cable_output = outputs.iter().find(|d| d.is_vbcable).map(|d| d.name.clone());

        let is_installed = cable_input.is_some() || cable_output.is_some();

        // Feedback warning: if both contain "cable"
        let feedback_warning = current_input.to_lowercase().contains("cable") && current_output.to_lowercase().contains("cable");

        VbCableStatus {
            is_installed,
            input_device_name: cable_input,
            output_device_name: cable_output,
            feedback_warning,
        }
    }

    pub fn find_input_device(&self, name: &str) -> Option<Device> {
        if name.is_empty() {
            return self.host.default_input_device();
        }
        if let Ok(devices) = self.host.input_devices() {
            for d in devices {
                if d.to_string() == name {
                    return Some(d);
                }
            }
        }
        self.host.default_input_device()
    }

    pub fn find_output_device(&self, name: &str) -> Option<Device> {
        if name.is_empty() {
            return self.host.default_output_device();
        }
        if let Ok(devices) = self.host.output_devices() {
            for d in devices {
                if d.to_string() == name {
                    return Some(d);
                }
            }
        }
        self.host.default_output_device()
    }

    pub fn is_cable_device(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.contains("cable") || lower.contains("vb-audio") || lower.contains("virtual audio")
    }

    fn get_supported_rates(device: &Device) -> Vec<u32> {
        let mut rates = Vec::new();
        if let Ok(configs) = device.supported_input_configs() {
            for c in configs {
                let min = c.min_sample_rate();
                let max = c.max_sample_rate();
                for r in [44100, 48000, 88200, 96000] {
                    if r >= min && r <= max && !rates.contains(&r) {
                        rates.push(r);
                    }
                }
            }
        }
        if let Ok(configs) = device.supported_output_configs() {
            for c in configs {
                let min = c.min_sample_rate();
                let max = c.max_sample_rate();
                for r in [44100, 48000, 88200, 96000] {
                    if r >= min && r <= max && !rates.contains(&r) {
                        rates.push(r);
                    }
                }
            }
        }
        rates.sort_unstable();
        if rates.is_empty() {
            vec![44100, 48000]
        } else {
            rates
        }
    }
}
