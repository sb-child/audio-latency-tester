use snafu::Snafu;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AppError {
    #[snafu(display("配置读取失败: {}", path.display()))]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("配置解析失败: {}", source))]
    ConfigParse { source: toml::de::Error },

    #[snafu(display("音频样本处理错误: {}", source))]
    SampleError { source: SampleError },

    #[snafu(display("音频后端错误: {}", source))]
    AudioError { source: AudioError },

    #[snafu(display("触发器错误: {}", source))]
    TriggerError { source: TriggerError },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SampleError {
    #[snafu(display("无法打开 WAV 文件: {}", path.display()))]
    WavOpen { path: PathBuf, source: hound::Error },

    #[snafu(display("文件: {}, 不支持的音频格式: {}", path.display(), desc))]
    UnsupportedFormat { path: PathBuf, desc: String },

    #[snafu(display("音频为空"))]
    Empty,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AudioError {
    #[snafu(display("不支持的后端: {}", backend))]
    UnsupportedBackend { backend: String },
    #[snafu(display("ALSA 设备初始化失败: {}", detail))]
    AlsaInit { detail: String },
    #[snafu(display("Pipewire 初始化失败: {}", detail))]
    PipewireInit { detail: String },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum TriggerError {
    #[snafu(display("无法打开输入设备 {}: {}", path.display(), source))]
    DeviceOpen {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("不支持的按键代码: {}", keycode))]
    UnsupportedKeycode { keycode: String },
    #[snafu(display("读取事件失败: {}", source))]
    EventRead { source: std::io::Error },
}
