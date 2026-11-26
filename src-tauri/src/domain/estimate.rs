use serde::Serialize;

use super::args::AUDIO_BITRATE_BPS;

const SAMPLES: usize = 3;
const SAMPLE_LEN_S: f64 = 3.0;
const WHOLE_VIDEO_BELOW_S: f64 = SAMPLES as f64 * SAMPLE_LEN_S * 1.5;
const CONTAINER_OVERHEAD: f64 = 1.01;

pub fn sample_windows(duration_s: f64) -> Vec<(f64, f64)> {
    if duration_s <= 0.0 {
        return Vec::new();
    }
    if duration_s <= WHOLE_VIDEO_BELOW_S {
        return vec![(0.0, duration_s)];
    }
    (0..SAMPLES)
        .map(|i| {
            let center = duration_s * (0.2 + 0.3 * i as f64);
            let start = (center - SAMPLE_LEN_S / 2.0).clamp(0.0, duration_s - SAMPLE_LEN_S);
            (start, SAMPLE_LEN_S)
        })
        .collect()
}

pub fn extrapolate(sample_bytes: u64, sample_s: f64, duration_s: f64, has_audio: bool) -> u64 {
    if sample_s <= 0.0 {
        return 0;
    }
    let video = sample_bytes as f64 / sample_s * duration_s;
    let audio = if has_audio {
        AUDIO_BITRATE_BPS as f64 / 8.0 * duration_s
    } else {
        0.0
    };
    ((video + audio) * CONTAINER_OVERHEAD).round() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeEstimate {
    pub bytes: u64,
    pub used_gpu: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_video_is_sampled_whole() {
        assert_eq!(sample_windows(10.0), vec![(0.0, 10.0)]);
        assert!(sample_windows(0.0).is_empty());
    }

    #[test]
    fn long_video_gets_three_spread_windows() {
        let w = sample_windows(100.0);
        assert_eq!(w, vec![(18.5, 3.0), (48.5, 3.0), (78.5, 3.0)]);
    }

    #[test]
    fn windows_stay_inside_video() {
        for d in [14.0, 15.0, 20.0, 37.3, 3600.0] {
            for (start, len) in sample_windows(d) {
                assert!(start >= 0.0 && start + len <= d + 1e-9, "d={d}");
            }
        }
    }

    #[test]
    fn extrapolates_video_and_audio() {
        let bytes = extrapolate(900_000, 9.0, 60.0, true);
        assert_eq!(bytes, ((6_000_000.0 + 960_000.0) * 1.01_f64).round() as u64);
        let silent = extrapolate(900_000, 9.0, 60.0, false);
        assert_eq!(silent, 6_060_000);
    }
}
