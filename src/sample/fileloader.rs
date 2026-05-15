use crate::{
    error::{SampleError, WavOpenSnafu},
    sample::Sample,
};
use resampler::{ResamplerFft, SampleRate};
use snafu::ResultExt;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

pub fn load_wav(filename: &str, rate: Option<SampleRate>) -> Result<Sample, SampleError> {
    info!("正在加载音频文件: {}", filename);
    let path = PathBuf::from(filename);

    let mut reader = hound::WavReader::open(&path).context(WavOpenSnafu { path: path.clone() })?;
    let wav_spec = reader.spec();
    let mut audio_data: Vec<f32> = Vec::new();
    if wav_spec.channels != 1 {
        return Err(SampleError::UnsupportedFormat {
            path: path.clone(),
            desc: format!("不支持 {} 声道, 要求 1", wav_spec.channels),
        });
    }
    let current_rate = match SampleRate::try_from(wav_spec.sample_rate) {
        Err(_) => {
            return Err(SampleError::UnsupportedFormat {
                path: path.clone(),
                desc: format!(
                    "不支持 {} 采样率, 要求 16000,22050,32000,44100,48000,88200,96000,176400,192000,384000",
                    wav_spec.sample_rate
                ),
            });
        }
        Ok(s) => s,
    };

    if wav_spec.sample_format == hound::SampleFormat::Int {
        match wav_spec.bits_per_sample {
            16 => {
                for sample in reader.samples::<i16>() {
                    let val = match sample {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("文件: {}, 损坏的帧: {}", &path.display(), e);
                            0
                        }
                    };
                    audio_data.push(val as f32 / i16::MAX as f32);
                }
            }
            32 => {
                for sample in reader.samples::<i32>() {
                    let val = match sample {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("文件: {}, 损坏的帧: {}", &path.display(), e);
                            0
                        }
                    };
                    audio_data.push(val as f32 / i32::MAX as f32);
                }
            }
            x => {
                return Err(SampleError::UnsupportedFormat {
                    path: path.clone(),
                    desc: format!("不支持 {} 位深度, 要求 16/32", x),
                });
            }
        }
    }
    if wav_spec.sample_format != hound::SampleFormat::Int {
        return Err(SampleError::UnsupportedFormat {
            path: path.clone(),
            desc: format!("不支持 {} 格式, 要求 Int", "Float"),
        });
    } else if audio_data.len() == 0 {
        return Err(SampleError::Empty);
    }
    info!("输入音频采样点: {}", audio_data.len());
    let mut final_rate = Into::<u32>::into(current_rate);
    if let Some(new_rate) = rate {
        if current_rate != new_rate {
            info!(
                "重采样音频: {} -> {}",
                Into::<u32>::into(current_rate),
                Into::<u32>::into(new_rate)
            );
            let mut resampler = ResamplerFft::new(2, current_rate, new_rate);
            let chunk_size_in = resampler.chunk_size_input();
            let chunk_size_out = resampler.chunk_size_output();
            let mut resampled_audio = Vec::new();
            for chunk in audio_data.chunks(chunk_size_in) {
                let mut input = vec![0.0f32; chunk_size_in];
                let mut output = vec![0.0f32; chunk_size_out];
                input[..chunk.len()].copy_from_slice(chunk);
                resampler.resample(&input, &mut output).unwrap();
                resampled_audio.extend_from_slice(&output);
            }
            audio_data = resampled_audio;
            final_rate = Into::<u32>::into(new_rate)
        }
    }
    info!("加载完成, 音频采样点: {}", audio_data.len());
    Ok(Sample::new(Arc::new(audio_data), final_rate))
}
