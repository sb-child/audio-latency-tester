#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// 触发音频播放
    Play,
    /// 停止当前播放（如果需要）
    Stop,
    /// 优雅退出
    Quit,
}
