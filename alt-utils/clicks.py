import sys
import csv
from pathlib import Path
import numpy as np
import scipy.signal as signal
from pydub import AudioSegment


def export_audacity_labels(results, txt_filename):
    try:
        with open(txt_filename, 'w', encoding='utf-8') as f_txt:
            for i, res in enumerate(results):
                a_time, b_time, distance = res
                f_txt.write(f"{b_time:.5f}\t{b_time:.5f}\tc{i+1}\n")
                f_txt.write(f"{a_time:.5f}\t{a_time:.5f}\tb{i+1}\n")
                f_txt.write(
                    f"{b_time:.5f}\t{a_time:.5f}\t{distance*1000:.2f}ms\n")
        print(f"Audacity标签已保存至: {txt_filename}", file=sys.stderr)
    except Exception as e:
        print(f"写入 {txt_filename} 失败: {e}", file=sys.stderr)


def print_grouped_distances(results, split_threshold=5.0):
    if not results:
        print("没有可输出的结果", file=sys.stderr)
        return
    current_group = []
    prev_a_time = results[0][0]
    for res in results:
        a_time, b_time, distance = res
        distance_ms_str = f"{distance * 1000:.5f}"
        if a_time - prev_a_time > split_threshold:
            print(",".join(current_group))
            current_group = []
        current_group.append(distance_ms_str)
        prev_a_time = a_time
    if current_group:
        print(",".join(current_group))


def analyze_audio(file_path):
    p = Path(file_path)
    if not p.is_file():
        print(f"{file_path} 不是文件", file=sys.stderr)
    parent = p.parent
    csv_filename = parent / (p.name + ".csv")
    txt_filename = parent / (p.name + "_labels.txt")
    print(f"加载音频文件: {file_path}", file=sys.stderr)
    try:
        audio = AudioSegment.from_mp3(file_path)
    except Exception as e:
        print(
            f"加载音频文件失败: {e}", file=sys.stderr)
        sys.exit(1)
    channels = audio.split_to_mono()
    if len(channels) > 1:
        right_channel = channels[1]
        print("检测到立体声，已提取右声道", file=sys.stderr)
    else:
        right_channel = channels[0]
        print("该音频为单声道，将直接处理该声道", file=sys.stderr)
    sr = right_channel.frame_rate
    samples = np.array(right_channel.get_array_of_samples(), dtype=np.float32)
    nperseg = 256
    step_samples = int(sr * 0.001)
    if step_samples <= 0:
        step_samples = 1
    noverlap = nperseg - step_samples
    if noverlap < 0:
        noverlap = nperseg // 2
    wave_len = 0.100
    print("正在进行 STFT", file=sys.stderr)
    f, t, Zxx = signal.stft(samples, fs=sr, nperseg=nperseg, noverlap=noverlap)
    magnitude = np.abs(Zxx)
    freq_mask = (f >= 1300) & (f <= 1700)
    energy = np.sum(magnitude[freq_mask, :], axis=0)
    dt = t[1] - t[0]
    window_len = max(1, int(wave_len / dt))
    rect_window = np.ones(window_len) / window_len
    smoothed_energy = np.convolve(energy, rect_window, mode='same')
    height_thresh = np.max(smoothed_energy) * 0.2
    distance_frames = int(0.200 / dt)
    peaks, _ = signal.find_peaks(
        smoothed_energy, height=height_thresh, distance=distance_frames)
    print(f"发现 {len(peaks)} 个蜂鸣声", file=sys.stderr)
    results = []
    nyq = 0.5 * sr
    cutoff = min(3500.0, nyq * 0.8)
    sos = signal.butter(4, cutoff / nyq, btype='highpass', output='sos')
    for p_idx in peaks:
        mid_time = t[p_idx]
        a_time = mid_time - (wave_len / 2)
        if a_time < 0:
            continue
        search_start = max(0.0, a_time - 0.100)
        start_idx = int(search_start * sr)
        end_idx = int(a_time * sr)
        if end_idx <= start_idx:
            continue
        segment = samples[start_idx:end_idx]
        if len(segment) > 33:
            try:
                filtered_segment = signal.sosfiltfilt(sos, segment)
            except ValueError:
                filtered_segment = segment
        else:
            filtered_segment = segment
        abs_sig = np.abs(filtered_segment)
        mean_val = np.mean(abs_sig)
        std_val = np.std(abs_sig)
        threshold = mean_val + 3 * std_val
        distance_samples = int(sr * 0.002)
        local_peaks, _ = signal.find_peaks(
            abs_sig, height=threshold, distance=distance_samples)
        if len(local_peaks) > 0:
            peak_idx_in_segment = local_peaks[0]
        else:
            peak_idx_in_segment = np.argmax(abs_sig)
        b_time = search_start + (peak_idx_in_segment / sr)
        distance = a_time - b_time
        results.append((a_time, b_time, distance))
    try:
        with open(csv_filename, 'w', newline='', encoding='utf-8') as f_csv:
            writer = csv.writer(f_csv)
            writer.writerow(['a_time', 'b_time', 'distance_b_to_a'])
            for res in results:
                writer.writerow(
                    [f"{res[0]:.5f}", f"{res[1]:.5f}", f"{res[2]:.5f}"])
        print(f"分析完成, 结果保存至: {csv_filename}", file=sys.stderr)
    except Exception as e:
        print(f"写入 {csv_filename} 失败: {e}", file=sys.stderr)
    export_audacity_labels(results, txt_filename)
    print_grouped_distances(results, split_threshold=5.0)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("使用说明: clicks.py <音频文件>", file=sys.stderr)
        sys.exit(1)
    audio_file = sys.argv[1]
    analyze_audio(audio_file)
