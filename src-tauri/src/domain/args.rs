use std::path::Path;

use super::model::{Encoder, Resolution, VideoInfo};
use super::quality::quality_param;

pub fn scale_filter(info: &VideoInfo, resolution: Resolution) -> Option<String> {
    if let Some(target) = resolution.target_short_side() {
        if target < info.short_side() {
            return Some(if info.is_portrait() {
                format!("scale={target}:-2")
            } else {
                format!("scale=-2:{target}")
            });
        }
    }
    if info.width % 2 != 0 || info.height % 2 != 0 {
        return Some("scale=trunc(iw/2)*2:trunc(ih/2)*2".to_string());
    }
    None
}

fn encoder_args(encoder: Encoder, quality: u8) -> Vec<String> {
    let q = quality_param(encoder, quality).to_string();
    let mut a: Vec<&str> = vec!["-c:v", encoder.ffmpeg_name()];
    match encoder {
        Encoder::X264 => a.extend(["-crf", &q, "-preset", "medium"]),
        Encoder::X265 => a.extend(["-crf", &q, "-preset", "medium", "-tag:v", "hvc1"]),
        Encoder::NvencH264 => a.extend(["-rc", "vbr", "-cq", &q, "-b:v", "0", "-preset", "p5"]),
        Encoder::NvencHevc => a.extend([
            "-rc", "vbr", "-cq", &q, "-b:v", "0", "-preset", "p5", "-tag:v", "hvc1",
        ]),
    }
    a.into_iter().map(String::from).collect()
}

pub const AUDIO_BITRATE_BPS: u64 = 128_000;

fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.map(String::from).to_vec()
}

const HDR_TO_SDR: &str = "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,\
tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,format=yuv420p";

pub fn video_filter(info: &VideoInfo, resolution: Resolution) -> Option<String> {
    let filters: Vec<String> = [
        scale_filter(info, resolution),
        info.hdr.then(|| HDR_TO_SDR.to_string()),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!filters.is_empty()).then(|| filters.join(","))
}

fn video_args(info: &VideoInfo, encoder: Encoder, quality: u8, resolution: Resolution) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(filter) = video_filter(info, resolution) {
        args.push("-vf".into());
        args.push(filter);
    }
    args.extend(encoder_args(encoder, quality));
    args.extend(strings(["-pix_fmt", "yuv420p"]));
    if info.hdr {
        args.extend(strings([
            "-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709",
        ]));
    }
    args
}

pub fn build(
    info: &VideoInfo,
    output: &Path,
    encoder: Encoder,
    quality: u8,
    resolution: Resolution,
) -> Vec<String> {
    let mut args = strings(["-hide_banner", "-v", "error", "-y", "-i"]);
    args.push(info.path.clone());
    args.extend(strings(["-map", "0:v:0", "-map", "0:a?", "-map_metadata", "0"]));
    args.extend(video_args(info, encoder, quality, resolution));
    args.extend(strings(["-c:a", "aac", "-b:a"]));
    args.push(format!("{}k", AUDIO_BITRATE_BPS / 1000));
    args.extend(strings(["-movflags", "+faststart", "-progress", "pipe:1", "-nostats"]));
    args.push(output.to_string_lossy().into_owned());
    args
}

pub fn build_sample(
    info: &VideoInfo,
    output: &Path,
    encoder: Encoder,
    quality: u8,
    resolution: Resolution,
    start_s: f64,
    len_s: f64,
) -> Vec<String> {
    let mut args = strings(["-hide_banner", "-v", "error", "-y", "-ss"]);
    args.push(format!("{start_s:.3}"));
    args.push("-t".into());
    args.push(format!("{len_s:.3}"));
    args.push("-i".into());
    args.push(info.path.clone());
    args.extend(strings(["-map", "0:v:0", "-an", "-sn", "-dn"]));
    args.extend(video_args(info, encoder, quality, resolution));
    args.extend(strings(["-f", "mp4"]));
    args.push(output.to_string_lossy().into_owned());
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn video(w: u32, h: u32) -> VideoInfo {
        VideoInfo {
            path: "in.mp4".into(),
            file_name: "in.mp4".into(),
            duration_s: 10.0,
            width: w,
            height: h,
            codec: "h264".into(),
            size_bytes: 1000,
            bitrate: None,
            has_audio: true,
            hdr: false,
        }
    }

    #[test]
    fn hdr_is_tonemapped_after_scaling() {
        let mut info = video(3840, 2160);
        info.hdr = true;
        let out = PathBuf::from("out.mp4");
        let a = build(&info, &out, Encoder::NvencHevc, 60, Resolution::P1080);
        let vf = value_after(&a, "-vf").unwrap();
        assert!(vf.starts_with("scale=-2:1080,zscale=t=linear"), "{vf}");
        assert!(vf.ends_with("format=yuv420p"));
        assert_eq!(value_after(&a, "-color_trc"), Some("bt709"));
    }

    #[test]
    fn sdr_has_no_tonemap() {
        let a = build(&video(1920, 1080), &PathBuf::from("o.mp4"), Encoder::X264, 60, Resolution::Original);
        assert!(!a.contains(&"-color_trc".to_string()));
        assert_eq!(video_filter(&video(1920, 1080), Resolution::Original), None);
    }

    fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args[i + 1].as_str())
    }

    #[test]
    fn landscape_downscale() {
        assert_eq!(
            scale_filter(&video(1920, 1080), Resolution::P720).as_deref(),
            Some("scale=-2:720")
        );
    }

    #[test]
    fn portrait_downscale_uses_width() {
        assert_eq!(
            scale_filter(&video(1080, 1920), Resolution::P720).as_deref(),
            Some("scale=720:-2")
        );
    }

    #[test]
    fn never_upscales() {
        assert_eq!(scale_filter(&video(1280, 720), Resolution::P1080), None);
        assert_eq!(scale_filter(&video(1280, 720), Resolution::P720), None);
    }

    #[test]
    fn odd_dimensions_are_evened() {
        assert_eq!(
            scale_filter(&video(1279, 719), Resolution::Original).as_deref(),
            Some("scale=trunc(iw/2)*2:trunc(ih/2)*2")
        );
    }

    #[test]
    fn x264_args() {
        let out = PathBuf::from("out.mp4");
        let a = build(&video(1920, 1080), &out, Encoder::X264, 60, Resolution::Original);
        assert_eq!(value_after(&a, "-c:v"), Some("libx264"));
        assert_eq!(value_after(&a, "-crf"), Some("24"));
        assert_eq!(value_after(&a, "-i"), Some("in.mp4"));
        assert_eq!(value_after(&a, "-progress"), Some("pipe:1"));
        assert!(!a.contains(&"-vf".to_string()));
        assert_eq!(a.last().map(String::as_str), Some("out.mp4"));
    }

    #[test]
    fn nvenc_hevc_args() {
        let out = PathBuf::from("out.mp4");
        let a = build(&video(1920, 1080), &out, Encoder::NvencHevc, 100, Resolution::P480);
        assert_eq!(value_after(&a, "-c:v"), Some("hevc_nvenc"));
        assert_eq!(value_after(&a, "-cq"), Some("21"));
        assert_eq!(value_after(&a, "-b:v"), Some("0"));
        assert_eq!(value_after(&a, "-tag:v"), Some("hvc1"));
        assert_eq!(value_after(&a, "-vf"), Some("scale=-2:480"));
        assert!(!a.contains(&"-crf".to_string()));
    }

    #[test]
    fn sample_args_match_final_video_settings() {
        let out = PathBuf::from("s.mp4");
        let info = video(1920, 1080);
        let a = build_sample(&info, &out, Encoder::X265, 60, Resolution::P720, 12.5, 3.0);
        assert_eq!(value_after(&a, "-ss"), Some("12.500"));
        assert_eq!(value_after(&a, "-t"), Some("3.000"));
        let pos = |f: &str| a.iter().position(|x| x == f).unwrap();
        assert!(pos("-ss") < pos("-i"));
        assert!(a.contains(&"-an".to_string()));
        assert_eq!(value_after(&a, "-crf"), Some("26"));
        assert_eq!(value_after(&a, "-vf"), Some("scale=-2:720"));
        assert!(!a.contains(&"-progress".to_string()));
    }
}
