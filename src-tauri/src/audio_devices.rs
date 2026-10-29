use serde::Serialize;

use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendDevice {
    pub id: String,
    pub name: String,
}

pub trait AudioDeviceBackend {
    fn input_devices(&self) -> Result<Vec<BackendDevice>, String>;
    fn output_devices(&self) -> Result<Vec<BackendDevice>, String>;
    fn default_input_id(&self) -> Result<Option<String>, String>;
    fn default_output_id(&self) -> Result<Option<String>, String>;
}

pub struct CpalAudioBackend {
    host: cpal::Host,
}

impl Default for CpalAudioBackend {
    fn default() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }
}

fn cpal_device(device: cpal::Device) -> Result<BackendDevice, String> {
    let id = device.id().map_err(|error| error.to_string())?.to_string();
    Ok(BackendDevice {
        id,
        name: device.to_string(),
    })
}

impl AudioDeviceBackend for CpalAudioBackend {
    fn input_devices(&self) -> Result<Vec<BackendDevice>, String> {
        self.host
            .input_devices()
            .map_err(|error| error.to_string())?
            .map(cpal_device)
            .collect()
    }

    fn output_devices(&self) -> Result<Vec<BackendDevice>, String> {
        self.host
            .output_devices()
            .map_err(|error| error.to_string())?
            .map(cpal_device)
            .collect()
    }

    fn default_input_id(&self) -> Result<Option<String>, String> {
        self.host
            .default_input_device()
            .map(|device| device.id().map(|id| id.to_string()))
            .transpose()
            .map_err(|error| error.to_string())
    }

    fn default_output_id(&self) -> Result<Option<String>, String> {
        self.host
            .default_output_device()
            .map(|device| device.id().map(|id| id.to_string()))
            .transpose()
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub direction: AudioDirection,
    pub is_default: bool,
}

pub struct AudioDeviceService<B> {
    backend: B,
}

impl<B> AudioDeviceService<B>
where
    B: AudioDeviceBackend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn devices(&self) -> Result<Vec<AudioDevice>, String> {
        let inputs = self.backend.input_devices().map_err(|_| {
            "Input audio devices could not be listed. Reconnect the device and retry.".to_owned()
        })?;
        let default_input = self.backend.default_input_id().map_err(|_| {
            "The default input device could not be read. Check Windows sound settings and retry."
                .to_owned()
        })?;
        let outputs = self.backend.output_devices().map_err(|_| {
            "Output audio devices could not be listed. Reconnect the device and retry.".to_owned()
        })?;
        let default_output = self.backend.default_output_id().map_err(|_| {
            "The default output device could not be read. Check Windows sound settings and retry."
                .to_owned()
        })?;

        let mut devices = Vec::with_capacity(inputs.len() + outputs.len());
        devices.extend(inputs.into_iter().map(|device| AudioDevice {
            is_default: default_input.as_deref() == Some(device.id.as_str()),
            id: device.id,
            name: device.name,
            direction: AudioDirection::Input,
        }));
        devices.extend(outputs.into_iter().map(|device| AudioDevice {
            is_default: default_output.as_deref() == Some(device.id.as_str()),
            id: device.id,
            name: device.name,
            direction: AudioDirection::Output,
        }));
        Ok(devices)
    }
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    AudioDeviceService::new(CpalAudioBackend::default()).devices()
}

#[cfg(test)]
mod tests {
    use super::{AudioDeviceBackend, AudioDeviceService, AudioDirection, BackendDevice};

    struct FakeBackend {
        fail_inputs: bool,
    }

    impl AudioDeviceBackend for FakeBackend {
        fn input_devices(&self) -> Result<Vec<BackendDevice>, String> {
            if self.fail_inputs {
                return Err("backend unavailable".to_owned());
            }
            Ok(vec![
                BackendDevice {
                    id: "mic-default".to_owned(),
                    name: "USB Audio".to_owned(),
                },
                BackendDevice {
                    id: "mic-second".to_owned(),
                    name: "USB Audio".to_owned(),
                },
            ])
        }

        fn output_devices(&self) -> Result<Vec<BackendDevice>, String> {
            Ok(vec![BackendDevice {
                id: "speakers".to_owned(),
                name: "Speakers".to_owned(),
            }])
        }

        fn default_input_id(&self) -> Result<Option<String>, String> {
            Ok(Some("mic-default".to_owned()))
        }

        fn default_output_id(&self) -> Result<Option<String>, String> {
            Ok(Some("speakers".to_owned()))
        }
    }

    #[test]
    fn separates_input_and_output_and_marks_defaults_by_stable_id() {
        let devices = AudioDeviceService::new(FakeBackend { fail_inputs: false })
            .devices()
            .expect("fake devices should enumerate");

        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0].direction, AudioDirection::Input);
        assert!(devices[0].is_default);
        assert_eq!(devices[1].direction, AudioDirection::Input);
        assert!(!devices[1].is_default);
        assert_eq!(devices[0].name, devices[1].name);
        assert_ne!(devices[0].id, devices[1].id);
        assert_eq!(devices[2].direction, AudioDirection::Output);
        assert!(devices[2].is_default);
    }

    #[test]
    fn names_the_failed_stage_and_safe_retry_when_enumeration_fails() {
        let error = AudioDeviceService::new(FakeBackend { fail_inputs: true })
            .devices()
            .expect_err("backend failure should be reported");

        assert_eq!(
            error,
            "Input audio devices could not be listed. Reconnect the device and retry."
        );
    }
}
