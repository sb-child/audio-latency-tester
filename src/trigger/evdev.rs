use crate::actor::AudioCommand;
use crate::config::TriggerConfig;
use crate::error::{DeviceOpenSnafu, EventReadSnafu, TriggerError};
use evdev::{Device, EventSummary, KeyCode};
use snafu::ResultExt;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use tracing::{debug, info};

// fn parse_keycode(name: &str) -> Result<KeyCode, TriggerError> {
//     match name.to_uppercase().as_str() {
//         "BTN_LEFT" => Ok(KeyCode::BTN_LEFT),
//         "BTN_RIGHT" => Ok(KeyCode::BTN_RIGHT),
//         "KEY_SPACE" => Ok(KeyCode::KEY_SPACE),
//         "KEY_Z" => Ok(KeyCode::KEY_Z),
//         "KEY_X" => Ok(KeyCode::KEY_X),
//         _ => UnsupportedKeycodeSnafu { keycode: name }.fail(),
//     }
// }

pub fn run_actor(config: TriggerConfig, tx: Sender<AudioCommand>) -> Result<(), TriggerError> {
    let path = PathBuf::from(&config.device);
    let mut input_device = Device::open(&path).context(DeviceOpenSnafu { path })?;
    // let target_key = parse_keycode(&config.keycode)?;
    let target_key: KeyCode = config.keycode.into();

    info!(
        "输入监听已启动。监听设备: {}, 触发按键: {:?}",
        config.device, target_key
    );

    loop {
        let events = input_device.fetch_events().context(EventReadSnafu)?;
        for event in events {
            if let EventSummary::Key(_, key, 1) = event.destructure() {
                if key == target_key {
                    debug!("检测到按键触发!");
                    if let Err(e) = tx.send(AudioCommand::Play) {
                        tracing::error!("无法向 Audio Actor 发送指令: {}", e);
                    }
                }
            }
        }
    }
}
