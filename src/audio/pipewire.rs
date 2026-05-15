use crate::actor::AudioCommand;
use crate::config::AudioConfig;
use crate::error::{AudioError, PipewireInitSnafu};
use crate::sample::Sample;
use pipewire as pw;
use pipewire::context::ContextBox;
use pipewire::main_loop::MainLoopBox;
use pipewire::spa::pod::Pod;
use pw::properties;
use pw::spa;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Once};
use std::thread;
use tracing::{error, info};

#[derive(Clone)]
struct AudioState {
    sample: Sample,
    is_playing: Arc<AtomicBool>,
    play_index: Arc<AtomicUsize>,
    buffer_size: Arc<AtomicU32>,
    autoconf: Arc<Once>,
}

pub fn run_actor(
    config: AudioConfig,
    sample: Sample,
    rx: Receiver<AudioCommand>,
) -> Result<(), AudioError> {
    let is_playing = Arc::new(AtomicBool::new(false));
    let play_index = Arc::new(AtomicUsize::new(0));
    let buffer_size = Arc::new(AtomicU32::new(config.buffer));

    let state = AudioState {
        sample,
        is_playing: is_playing.clone(),
        play_index: play_index.clone(),
        buffer_size,
        autoconf: Arc::new(Once::new()),
    };

    thread::spawn(move || {
        if let Err(e) = run_pw_thread(state) {
            error!("Pipewire 音频后端错误: {}", e);
        }
    });

    info!("Pipewire 已挂载完毕");

    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCommand::Play => {
                play_index.store(0, Ordering::SeqCst);
                is_playing.store(true, Ordering::SeqCst);
                info!("Pipewire 触发播放");
            }
            AudioCommand::Stop => {
                is_playing.store(false, Ordering::SeqCst);
            }
            AudioCommand::Quit => break,
        }
    }

    Ok(())
}

fn run_pw_thread(state: AudioState) -> Result<(), AudioError> {
    pw::init();

    let mainloop = MainLoopBox::new(None).map_err(|e| {
        PipewireInitSnafu {
            detail: format!("无法创建 PipeWire MainLoop: {}", e),
        }
        .build()
    })?;

    let context = ContextBox::new(&mainloop.loop_(), None).map_err(|e| {
        PipewireInitSnafu {
            detail: format!("无法创建 Context: {}", e),
        }
        .build()
    })?;

    let core = context.connect(None).map_err(|e| {
        PipewireInitSnafu {
            detail: format!("无法连接到 PipeWire Core: {}", e),
        }
        .build()
    })?;

    let props = properties::properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::MEDIA_CATEGORY => "Playback",
        *pw::keys::MEDIA_ROLE => "Game",
        *pw::keys::NODE_NAME => "audio-latency-tester-pw",
        *pw::keys::NODE_LATENCY => format!("{}/{}", state.buffer_size.load(Ordering::Relaxed), state.sample.get_rate()),
        *pw::keys::AUDIO_CHANNELS => "2",
    };

    let stream = pw::stream::StreamBox::new(&core, "stream", props).map_err(|e| {
        PipewireInitSnafu {
            detail: format!("无法创建 Stream: {}", e),
        }
        .build()
    })?;

    let _listener = stream
        .add_local_listener_with_user_data(state.clone())
        .process(|stream, user_data| match stream.dequeue_buffer() {
            None => return,
            Some(mut buffer) => {
                user_data.autoconf.clone().call_once(|| {
                    if let Some(lat) = stream.properties().get("node.latency") {
                        info!("当前 node.latency = {}", lat);
                        if let Some(Ok(quantum)) =
                            lat.split_once("/").map(|(x, _)| x.parse::<u32>())
                        {
                            info!("自动设置缓冲区为: {}", quantum);
                            user_data.buffer_size.store(quantum, Ordering::Relaxed);
                        }
                    }
                });

                let datas = buffer.datas_mut();
                if datas.is_empty() {
                    return;
                }
                let data = &mut datas[0];
                let stride = std::mem::size_of::<f32>() * 2;

                if let Some(slice) = data.data() {
                    let requested_frames = user_data.buffer_size.load(Ordering::Relaxed) as usize;
                    let out = unsafe {
                        std::slice::from_raw_parts_mut(
                            slice.as_mut_ptr() as *mut f32,
                            requested_frames * 2,
                        )
                    };

                    for frame in out.chunks_mut(2) {
                        if user_data.is_playing.load(Ordering::Acquire) {
                            let idx = user_data.play_index.load(Ordering::Relaxed);
                            let sample_data = user_data.sample.get_data();
                            if idx < sample_data.len() {
                                let sample = sample_data[idx];
                                frame[0] = sample;
                                frame[1] = sample;
                                user_data.play_index.store(idx + 1, Ordering::Relaxed);
                            } else {
                                user_data.is_playing.store(false, Ordering::Release);
                                frame[0] = 0.0;
                                frame[1] = 0.0;
                            }
                        } else {
                            frame[0] = 0.0;
                            frame[1] = 0.0;
                        }
                    }

                    let chunk = data.chunk_mut();
                    *chunk.size_mut() = (requested_frames * stride) as u32;
                    *chunk.stride_mut() = stride as i32;
                }
            }
        })
        .register()
        .map_err(|e| {
            PipewireInitSnafu {
                detail: format!("无法注册 Stream Listener: {}", e),
            }
            .build()
        })?;

    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaType,
            Id,
            pw::spa::param::format::MediaType::Audio
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pw::spa::param::format::MediaSubtype::Raw
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::AudioFormat,
            Id,
            pw::spa::param::audio::AudioFormat::F32LE
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::AudioRate,
            Int,
            // not possible to panic
            state.sample.get_rate().try_into().unwrap()
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::AudioChannels,
            Int,
            2
        )
    );

    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .map_err(|e| {
        PipewireInitSnafu {
            detail: format!("参数序列化失败: {:?}", e),
        }
        .build()
    })?
    .0
    .into_inner();

    let pod = Pod::from_bytes(&values).ok_or_else(|| {
        PipewireInitSnafu {
            detail: "无法从序列化字节构建 Pod 参数".to_string(),
        }
        .build()
    })?;

    let mut params = [pod];

    stream
        .connect(
            spa::utils::Direction::Output,
            None,
            pw::stream::StreamFlags::AUTOCONNECT
                | pw::stream::StreamFlags::MAP_BUFFERS
                | pw::stream::StreamFlags::RT_PROCESS,
            &mut params,
        )
        .map_err(|e| {
            PipewireInitSnafu {
                detail: format!("Stream 连接失败: {}", e),
            }
            .build()
        })?;

    info!("Pipewire 主循环已启动");
    mainloop.run();

    Ok(())
}
