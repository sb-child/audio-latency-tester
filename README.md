# Audio Latency Tester

音频延迟测试工具，<sub>调校</sub>调教音游痴们用的。

## how to use

对系统的要求: Linux, 配置了 Pipewire/ALSA 音频服务。

看repo里有个 `config.toml`，那是示例配置文件，里面有说明。你可以拷走一份改名比如 `local_profile1.toml` 然后调节里面的参数。

在终端输入 `cargo b -r` 编译代码，产物位于 `target/release/audio-latency-tester`，你可以把它拷走拿到别的地方用。

可以直接执行 `audio-latency-tester` 程序，它默认会找当前目录(working directory)下的 `config.toml` 文件。

配置文件不在这怎么办? 你可以给程序指定参数比如 `-c local_profile1.toml` 或 `-c /tmp/wtf.toml` 要求它去读取指定文件。

一切准备就绪，日志都是绿的，你按下触发键，扬声器立刻播放尖锐的声波。that's all.

## why

这是我第3次试图搞清楚 [osu!lazer](https://github.com/ppy/osu) 在我系统上的音频滞后的根源。

上次是2025年10月~11月。我认为延迟出在可能不公平的任务调度上，然后我瞎几把调把音频服务饿死了：

> System audio playback is not working as expected. Some online functionality will not work. Please check your audio drivers.
> Score submission cancelled due to audio playback rate discrepancy.

但是我获得了一些[成果](https://github.com/sb-child/notes/tree/main/osu#pipewire-%E4%B8%8E%E9%9F%B3%E9%A2%91%E5%BB%B6%E8%BF%9F)，发现这简直是采样率对不上，缓冲区太大的问题。

再上次就是2025年7月。我信誓旦旦的把采样率锁到384k因为我的声卡足够hifi，但是怎么折腾都还有50ms的延迟。

## result

本次测试运行在搭载**AMD Ryzen 5 4600U**的Fedora笔记本上。

- 声卡**iBasso Macaron**直连主机。数字滤波器**Short delay Fast roll off**。耳机型号**Sennheiser HD600**。
- 鼠标**logitech G502 Hero**通过USB Hub连接主机。触发键为鼠标左键。后端 `evdev`。
- 录音设备，手机距离扬声器和鼠标5cm内。
- 音频后端: **pipewire**: (`native`), **alsa**: (`cpal` -> `pipewire-alsa`)

| 音频后端 | 客户端设置         | 输出设备设置         | 实测延迟(ms) n=10    |
| -------- | ------------------ | -------------------- | -------------------- |
| pipewire | 512/44100 F32LE    | 32/44100 S32LE       | avg=20.1 sd=3.51     |
| pipewire | 256/44100 F32LE    | 32/44100 S32LE       | avg=17.6 sd=1.65     |
| pipewire | 64/44100 F32LE     | 32/44100 S32LE       | avg=14.8 sd=1.32     |
| pipewire | **32/44100 F32LE** | 32/44100 S32LE       | **avg=15.0 sd=0.67** |
| alsa     | 512/44100 F32LE    | 32/44100 S32LE       | avg=30.4 sd=3.41     |
| alsa     | 256/44100 F32LE    | 32/44100 S32LE       | avg=26.1 sd=0.99     |
| alsa     | 64/44100 F32LE     | 32/44100 S32LE       | avg=17.2 sd=0.63     |
| alsa     | **58/44100 F32LE** | 32/44100 S32LE       | **avg=16.2 sd=0.63** |
| pipewire | 32/44100 F32LE     | 32/48000 S32LE       | avg=14.9 sd=0.88     |
| pipewire | 32/44100 F32LE     | 64/96000 S32LE       | avg=12.0 sd=0.67     |
| pipewire | 32/44100 F32LE     | 128/192000 S32LE     | avg=10.0 sd=0.67     |
| pipewire | 32/44100 F32LE     | **256/384000 S32LE** | **avg=9.5 sd=0.53**  |

为什么没有用板载声卡？因为它播放100毫秒的音频很吃力会逐渐变成锯齿声，虽然它的最低延迟实测不大于5ms。

接下来是 **osu!lazer**。

- 版本: **2026.513.0-tachyon**。环境变量 `OSU_SDL3=true`。
- 音频后端: `BASS` -> `pipewire-alsa`。
- 窗管: **niri**，禁用 xwayland。
- 测量鼠标按键按下到 Hit Sound 播放的间隔。
- 作为热身，在测量结果中排除第一个note。

| `PIPEWIRE_LATENCY` | 输出设备设置   | 实测延迟(ms) n=55 |
| ------------------ | -------------- | ----------------- |
| 64/44100           | 64/44100 S32LE | avg=39.95 sd=3.08 |
| 256/44100          | 64/44100 S32LE | avg=38.69 sd=3.11 |
| 512/44100          | 64/44100 S32LE | avg=38.89 sd=2.94 |
| 未指定(441/44100)  | 64/44100 S32LE | avg=38.93 sd=3.01 |

好像 `PIPEWIRE_LATENCY` 什么都没做一样，虽然 `pw-top` 如预期显示。

| 输出设备设置     | 实测延迟(ms) n=55 |
| ---------------- | ----------------- |
| 64/96000 S32LE   | avg=30.62 sd=3.29 |
| 128/192000 S32LE | avg=29.49 sd=3.53 |
| 256/384000 S32LE | avg=29.25 sd=3.34 |

未指定 `PIPEWIRE_LATENCY`，调高输出设备的采样率，只能降低约8ms延迟。
