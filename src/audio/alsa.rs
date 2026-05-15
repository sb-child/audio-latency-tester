use crate::actor::AudioCommand;
use crate::config::AudioConfig;
use crate::error::{AlsaInitSnafu, AudioError};
use crate::sample::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SupportedBufferSize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use tracing::{error, info, warn};

pub fn run_actor(
    config: AudioConfig,
    sample: Sample,
    rx: Receiver<AudioCommand>,
) -> Result<(), AudioError> {
    let host = cpal::host_from_id(cpal::HostId::Alsa).map_err(|e| {
        AlsaInitSnafu {
            detail: format!("Host 获取失败: {}", e),
        }
        .build()
    })?;

    let device = host.default_output_device().ok_or_else(|| {
        AlsaInitSnafu {
            detail: "找不到默认音频输出设备".to_string(),
        }
        .build()
    })?;

    let default_config = device.default_output_config().map_err(|e| {
        AlsaInitSnafu {
            detail: format!("无法获取配置: {}", e),
        }
        .build()
    })?;

    println!("默认音频配置: {:?}", default_config);
    let device_id = device.id().map(|x| x.to_string());
    let device_desc = device.description().map(|x| x.to_string());
    info!("音频输出设备: id: {:?}, desc: {:?}", device_id, device_desc);

    // if default_config.sample_rate() != 44100 {
    //     return Err(AlsaInitSnafu {
    //         detail: format!("音频采样率 {} 必须等于 44100", default_config.sample_rate()),
    //     }
    //     .build());
    // }

    let mut stream_config = default_config.config();
    let default_sample_rate = default_config.sample_rate();
    let data_sample_rate = sample.get_rate();
    info!(
        "默认采样率: {}, 已设为: {}",
        default_sample_rate, data_sample_rate
    );
    stream_config.sample_rate = data_sample_rate;
    match default_config.buffer_size() {
        SupportedBufferSize::Range { min, max } => {
            info!(
                "支持的缓冲区范围: {} - {}, 正在设置为 {}",
                min, max, config.buffer
            );
            let target_buffer = config.buffer.clamp(*min, *max);
            stream_config.buffer_size = BufferSize::Fixed(target_buffer);
        }
        SupportedBufferSize::Unknown => {
            warn!("无法查询缓冲区设置");
        }
    };

    info!("最终缓冲区大小: {:?}", stream_config.buffer_size);

    let is_playing = Arc::new(AtomicBool::new(false));
    let play_index = Arc::new(AtomicUsize::new(0));

    let stream = build_stream(
        &device,
        &stream_config,
        sample.get_data(),
        is_playing.clone(),
        play_index.clone(),
    )?;

    stream.play().map_err(|e| {
        AlsaInitSnafu {
            detail: format!("播放失败: {}", e),
        }
        .build()
    })?;
    info!("Alsa 音频流已挂载完毕");

    // Actor 消息循环
    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCommand::Play => {
                play_index.store(0, Ordering::SeqCst);
                is_playing.store(true, Ordering::SeqCst);
                info!("Alsa 触发播放");
            }
            AudioCommand::Stop => {
                is_playing.store(false, Ordering::SeqCst);
            }
            AudioCommand::Quit => break,
        }
    }

    Ok(())
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    raw_data: Arc<Vec<f32>>,
    is_playing: Arc<AtomicBool>,
    play_index: Arc<AtomicUsize>,
) -> Result<cpal::Stream, AudioError> {
    let channels = config.channels as usize;
    let err_fn = |err| error!("音频流错误: {}", err);

    device
        .build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    if is_playing.load(Ordering::Acquire) {
                        let idx = play_index.load(Ordering::Relaxed);
                        if idx < raw_data.len() {
                            let sample_value = raw_data[idx];
                            for channel in frame.iter_mut() {
                                *channel = sample_value;
                            }
                            play_index.store(idx + 1, Ordering::Relaxed);
                        } else {
                            is_playing.store(false, Ordering::Release);
                            frame.fill(0.0);
                        }
                    } else {
                        frame.fill(0.0);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| {
            AlsaInitSnafu {
                detail: format!("构建流失败: {}", e),
            }
            .build()
        })
}
