use std::{sync::mpsc, thread};

use snafu::ResultExt;
use tracing::{info, warn};

use crate::{
    actor, audio,
    config::{AppConfig, AudioBackend, SampleBackend, TriggerBackend},
    error::{AppError, AudioSnafu, SampleSnafu},
    sample, trigger,
};

pub fn app(app_config: AppConfig) -> Result<(), AppError> {
    info!("初始化样本后端: {:?}", app_config.sample.backend);
    let sample_data = match app_config.sample.backend {
        SampleBackend::FileLoader => sample::fileloader::load_wav(
            &app_config.sample.filename,
            app_config.sample.rate.map(|x| x.into()),
        )
        .context(SampleSnafu)?,
    };

    let (trig_tx, trig_rx) = mpsc::channel::<actor::AudioCommand>();
    let trigger_config = app_config.trigger.clone();
    info!("初始化触发器后端: {:?}", trigger_config.backend);
    let (trig_ret_tx, _trig_ret_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = match trigger_config.backend {
            TriggerBackend::Evdev => trigger::evdev::run_actor(trigger_config, trig_tx),
        };
        if let Err(e) = result {
            warn!("触发器后端错误: {}", e);
            let _ = trig_ret_tx.send(e);
        }
    });
    // todo: trig_ret_rx

    tracing::info!("初始化音频后端: {:?}", app_config.audio.backend);
    match app_config.audio.backend {
        AudioBackend::Alsa => {
            audio::alsa::run_actor(app_config.audio, sample_data, trig_rx).context(AudioSnafu)?;
        }
        AudioBackend::Pipewire => {
            audio::pipewire::run_actor(app_config.audio, sample_data, trig_rx)
                .context(AudioSnafu)?;
        }
    }
    Ok(())
}
