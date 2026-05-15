use std::str::FromStr;

use evdev::KeyCode;
use resampler::SampleRate;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioBackend {
    Alsa,
    Pipewire,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SampleBackend {
    FileLoader,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TriggerBackend {
    Evdev,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub sample: SampleConfig,
    pub trigger: TriggerConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AudioConfig {
    pub backend: AudioBackend,
    pub buffer: u32,
}

#[derive(Debug, Clone)]
pub struct SampleRateWrap(SampleRate);

impl<'de> Deserialize<'de> for SampleRateWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = u32::deserialize(deserializer)?;
        Ok(SampleRateWrap(SampleRate::try_from(s).map_err(|_| {
            serde::de::Error::invalid_type(
                // 对对，因为 SampleRate::try_from 的 Err 里没东西
                serde::de::Unexpected::Unsigned(s as u64),
                &"one of 16000,22050,32000,44100,48000,88200,96000,176400,192000,384000",
            )
        })?))
    }
}

impl Into<SampleRate> for SampleRateWrap {
    fn into(self) -> SampleRate {
        self.0
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SampleConfig {
    pub backend: SampleBackend,
    pub filename: String,
    pub rate: Option<SampleRateWrap>,
}

#[derive(Debug, Clone)]
pub struct KeyCodeWrap(KeyCode);

impl<'de> Deserialize<'de> for KeyCodeWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum KeyCodeHelper {
            Num(u16),
            Str(String),
        }
        match KeyCodeHelper::deserialize(deserializer)? {
            KeyCodeHelper::Num(v) => Ok(KeyCodeWrap(KeyCode::new(v))),
            KeyCodeHelper::Str(v) => KeyCode::from_str(&v).map(KeyCodeWrap).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(&v), &"one of keycodes")
            }),
        }
    }
}

impl Into<KeyCode> for KeyCodeWrap {
    fn into(self) -> KeyCode {
        self.0
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TriggerConfig {
    pub backend: TriggerBackend,
    pub keycode: KeyCodeWrap,
    pub device: String,
}
