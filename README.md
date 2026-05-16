# Audio Latency Tester

音频延迟测试工具，<sub>调校</sub>调教音游痴们用的。

## how to use

对系统的要求: Linux, 配置了 Pipewire/ALSA 音频服务。

看repo里有个 `config.toml`，那是示例配置文件，里面有说明。你可以拷走一份改名比如 `local_profile1.toml` 然后调节里面的参数。

在终端输入 `cargo b -r` 编译代码，产物位于 `target/release/audio-latency-tester`，你可以把它拷走拿到别的地方用。

可以直接执行 `audio-latency-tester` 程序，它默认会找当前目录(working directory)下的 `config.toml` 文件。

配置文件不在这怎么办? 你可以给程序指定参数比如 `-c local_profile1.toml` 或 `-c /tmp/wtf.toml` 要求它去读取指定文件。

一切准备就绪，日志都是绿的，你按下触发键，扬声器立刻播放尖锐的声波。that's all.

你录音之后可以用 `alt-utils/clicks.py` 节省你宝贵的眼睛和时间，再用 `alt-utils/avg.py` 算出你设备的水平。

## why

这是我第3次试图搞清楚 [osu!lazer](https://github.com/ppy/osu) 在我系统上的音频滞后的根源。

上次是2025年10月~11月。我认为延迟出在可能不公平的任务调度上，然后我瞎几把调把音频服务饿死了：

> System audio playback is not working as expected. Some online functionality will not work. Please check your audio drivers.
> Score submission cancelled due to audio playback rate discrepancy.

但是我获得了一些[成果](https://github.com/sb-child/notes/tree/main/osu#pipewire-%E4%B8%8E%E9%9F%B3%E9%A2%91%E5%BB%B6%E8%BF%9F)，发现这简直是采样率对不上，缓冲区太大的问题。

再上次就是2025年7月。我信誓旦旦的把采样率锁到384k因为我的声卡足够hifi，但是怎么折腾都还有50ms的延迟。

## result

本次测试运行在搭载**AMD Ryzen 5 4600U**的Fedora笔记本上。

- 程序: **audio-latency-tester**
- 声卡**iBasso Macaron**直连主机。数字滤波器**Short delay Fast roll off**。耳机型号**Sennheiser HD600**。
- 鼠标**logitech G502 Hero**通过USB Hub连接主机。触发键为鼠标左键。后端 `evdev`。
- 录音设备，手机距离扬声器和鼠标5cm内。
- 音频后端: **pipewire**: (`native`), **alsa**: (`cpal` -> `pipewire-alsa`)

| 音频后端 | 客户端设置         | 声卡设置             | 实测延迟(ms) n=10    |
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

- 程序: **osu! 2026.513.0-tachyon**。环境变量 `OSU_SDL3=true`。
- 音频后端: `BASS` -> `pipewire-alsa`。
- 窗管: **niri**，禁用 xwayland。
- 测量鼠标按键按下到 Hit Sound 播放的间隔。
- 作为热身，在测量结果中排除第一个note。

| `PIPEWIRE_LATENCY` | 声卡设置       | 实测延迟(ms) n=55 |
| ------------------ | -------------- | ----------------- |
| 64/44100           | 64/44100 S32LE | avg=39.95 sd=3.08 |
| 256/44100          | 64/44100 S32LE | avg=38.69 sd=3.11 |
| 512/44100          | 64/44100 S32LE | avg=38.89 sd=2.94 |
| 未指定(441/44100)  | 64/44100 S32LE | avg=38.93 sd=3.01 |

好像 `PIPEWIRE_LATENCY` 什么都没做一样，虽然 `pw-top` 如预期显示。

| 声卡设置         | 实测延迟(ms) n=55 |
| ---------------- | ----------------- |
| 64/96000 S32LE   | avg=30.62 sd=3.29 |
| 128/192000 S32LE | avg=29.49 sd=3.53 |
| 256/384000 S32LE | avg=29.25 sd=3.34 |

未指定 `PIPEWIRE_LATENCY`，调高输出设备的采样率，只能降低约8ms延迟。

> 以上结果都是我对着Audacity挨个量的。然后接下来让 `alt-utils` 帮我，通常比人工标注多出1.5ms，但是它可复现。

| 声卡设置         | 实测延迟(ms) n=55 |
| ---------------- | ----------------- |
| 64/44100 S32LE   | avg=41.23 sd=3.19 |
| 64/48000 S32LE   | avg=40.42 sd=3.28 |
| 64/88200 S32LE   | avg=32.44 sd=3.26 |
| 64/96000 S32LE   | avg=31.12 sd=4.18 |
| 128/176400 S32LE | avg=30.46 sd=3.41 |
| 128/192000 S32LE | avg=30.15 sd=3.58 |
| 256/352800 S32LE | avg=31.09 sd=3.17 |
| 256/384000 S32LE | avg=30.16 sd=3.42 |

现在osu卡在了30ms的瓶颈。**那换用pro-audio模式就能改善音频延迟吗？**

```bash
# 设置为模拟输出。声卡标识变为 alsa_output.usb-iBasso_Macaron-01.analog-stereo
$ pactl set-card-profile alsa_card.usb-iBasso_Macaron-01 output:analog-stereo
# 设置为专业音频。声卡标识变为 alsa_output.usb-iBasso_Macaron-01.pro-output-0
$ pactl set-card-profile alsa_card.usb-iBasso_Macaron-01 pro-audio
```

- 程序: **audio-latency-tester**。
- 音频后端: **pipewire**。
- 声卡设置: **64/44100 S32LE**。

| 客户端设置         | 声卡模式          | 实测延迟(ms) n=20     |
| ------------------ | ----------------- | --------------------- |
| 512/44100 F32LE    | pro-audio         | avg=27.00 sd=4.08     |
| 256/44100 F32LE    | pro-audio         | avg=24.51 sd=1.62     |
| 64/44100 F32LE     | pro-audio         | avg=22.30 sd=1.13     |
| **32/44100 F32LE** | **pro-audio**     | **avg=22.29 sd=0.87** |
| 512/44100 F32LE    | analog-stereo     | avg=28.22 sd=4.07     |
| 256/44100 F32LE    | analog-stereo     | avg=24.52 sd=1.94     |
| 64/44100 F32LE     | analog-stereo     | avg=22.58 sd=1.61     |
| **32/44100 F32LE** | **analog-stereo** | **avg=22.49 sd=1.22** |

音频服务器这边收效甚微。那声卡的极限呢？

- 客户端设置: **64/48000**。

| 声卡设置             | 声卡模式          | 实测延迟(ms) n=20     |
| -------------------- | ----------------- | --------------------- |
| 32/48000 F32LE       | pro-audio         | avg=15.76 sd=0.73     |
| 64/96000 F32LE       | pro-audio         | avg=13.19 sd=0.80     |
| 128/192000 F32LE     | pro-audio         | avg=11.58 sd=0.72     |
| **256/384000 F32LE** | **pro-audio**     | **avg=10.65 sd=0.62** |
| 32/48000 F32LE       | analog-stereo     | avg=15.77 sd=0.95     |
| 64/96000 F32LE       | analog-stereo     | avg=13.01 sd=0.90     |
| 128/192000 F32LE     | analog-stereo     | avg=11.83 sd=1.41     |
| **256/384000 F32LE** | **analog-stereo** | **avg=10.45 sd=0.80** |
