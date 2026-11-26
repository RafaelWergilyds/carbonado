use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use tauri::AppHandle;

use super::sidecar;
use crate::domain::model::VideoInfo;
use crate::domain::progress::ProgressParser;
use crate::error::{AppError, AppResult, InputProblem};

#[derive(Deserialize)]
struct ProbeOutput {
    #[serde(default)]
    streams: Vec<ProbeStream>,
    format: ProbeFormat,
}

#[derive(Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<String>,
    color_transfer: Option<String>,
    #[serde(default)]
    side_data_list: Vec<SideData>,
    #[serde(default)]
    tags: HashMap<String, String>,
    #[serde(default)]
    disposition: HashMap<String, i64>,
}

#[derive(Deserialize)]
struct SideData {
    rotation: Option<f64>,
}

#[derive(Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

impl ProbeStream {
    fn is_cover_art(&self) -> bool {
        self.disposition.get("attached_pic").copied().unwrap_or(0) == 1
    }

    fn rotation(&self) -> i64 {
        self.side_data_list
            .iter()
            .find_map(|s| s.rotation)
            .map(|r| r.round() as i64)
            .or_else(|| self.tags.get("rotate").and_then(|r| r.parse().ok()))
            .unwrap_or(0)
    }

    fn is_hdr(&self) -> bool {
        matches!(self.color_transfer.as_deref(), Some("smpte2084" | "arib-std-b67"))
    }

    fn duration(&self) -> Option<f64> {
        let field = self.duration.as_deref().and_then(|d| d.parse::<f64>().ok());
        let tag = || {
            self.tags
                .iter()
                .find(|(k, _)| k.to_ascii_uppercase().starts_with("DURATION"))
                .and_then(|(_, v)| parse_clock(v))
        };
        field.filter(|d| *d > 0.0).or_else(tag).filter(|d| *d > 0.0)
    }
}

pub fn parse_clock(value: &str) -> Option<f64> {
    let mut parts = value.trim().rsplit(':');
    let seconds: f64 = parts.next()?.parse().ok()?;
    let minutes: f64 = parts.next().map_or(Ok(0.0), str::parse).ok()?;
    let hours: f64 = parts.next().map_or(Ok(0.0), str::parse).ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

pub fn parse(json: &str, path: &str, size_bytes: Option<u64>) -> AppResult<VideoInfo> {
    let out: ProbeOutput =
        serde_json::from_str(json).map_err(|e| AppError::ProbeFailed(e.to_string()))?;

    let video = out
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video") && !s.is_cover_art())
        .ok_or_else(|| AppError::InvalidInput(InputProblem::NoVideoStream))?;

    let (mut width, mut height) = match (video.width, video.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w, h),
        _ => return Err(AppError::ProbeFailed("missing video dimensions".into())),
    };
    if video.rotation().rem_euclid(180) == 90 {
        std::mem::swap(&mut width, &mut height);
    }

    let duration_s = out
        .format
        .duration
        .as_deref()
        .and_then(|d| d.parse::<f64>().ok())
        .filter(|d| *d > 0.0)
        .or_else(|| video.duration())
        .or_else(|| out.streams.iter().find_map(ProbeStream::duration))
        .unwrap_or(0.0);

    let size_bytes = size_bytes
        .or_else(|| out.format.size.as_deref().and_then(|s| s.parse().ok()))
        .unwrap_or(0);

    let file_name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    Ok(VideoInfo {
        path: path.to_string(),
        file_name,
        duration_s,
        width,
        height,
        codec: video.codec_name.clone().unwrap_or_else(|| "?".into()),
        size_bytes,
        bitrate: out.format.bit_rate.as_deref().and_then(|b| b.parse().ok()),
        has_audio: out.streams.iter().any(|s| s.codec_type.as_deref() == Some("audio")),
        hdr: video.is_hdr(),
    })
}

async fn scan_duration(app: &AppHandle, path: &str) -> Option<f64> {
    let output = sidecar::ffmpeg(app)
        .ok()?
        .args([
            "-hide_banner", "-v", "error",
            "-i", path,
            "-map", "0:v:0", "-c", "copy", "-f", "null",
            "-progress", "pipe:1", "-nostats", "-",
        ])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut parser = ProgressParser::default();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| parser.feed(line))
        .last()
        .map(|snap| snap.out_time_s)
        .filter(|d| *d > 0.0)
}

pub async fn probe(app: &AppHandle, path: &str) -> AppResult<VideoInfo> {
    let meta = std::fs::metadata(path)
        .map_err(|_| AppError::InvalidInput(InputProblem::FileNotFound))?;
    if !meta.is_file() {
        return Err(AppError::InvalidInput(InputProblem::NotAFile));
    }

    let output = sidecar::ffprobe(app)?
        .args([
            "-v", "error",
            "-print_format", "json",
            "-show_format", "-show_streams",
            path,
        ])
        .output()
        .await
        .map_err(|e| AppError::FfmpegNotFound(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AppError::ProbeFailed(stderr));
    }
    let mut info = parse(&String::from_utf8_lossy(&output.stdout), path, Some(meta.len()))?;
    if info.duration_s <= 0.0 {
        info.duration_s = scan_duration(app, path)
            .await
            .ok_or_else(|| AppError::InvalidInput(InputProblem::UnknownDuration))?;
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "streams": [
        { "index": 0, "codec_name": "mjpeg", "codec_type": "video", "width": 300, "height": 300,
          "disposition": { "default": 0, "attached_pic": 1 } },
        { "index": 1, "codec_name": "h264", "codec_type": "video", "width": 1920, "height": 1080,
          "disposition": { "default": 1, "attached_pic": 0 },
          "side_data_list": [ { "side_data_type": "Display Matrix", "rotation": -90 } ] },
        { "index": 2, "codec_name": "aac", "codec_type": "audio" }
      ],
      "format": { "filename": "C:/v/clip.mp4", "duration": "20.040000", "size": "52428800", "bit_rate": "20930000" }
    }"#;

    #[test]
    fn parses_sample_skipping_cover_art_and_applying_rotation() {
        let info = parse(SAMPLE, "C:/v/clip.mp4", None).unwrap();
        assert_eq!(info.file_name, "clip.mp4");
        assert_eq!(info.codec, "h264");
        assert_eq!((info.width, info.height), (1080, 1920));
        assert!(info.is_portrait());
        assert_eq!(info.duration_s, 20.04);
        assert_eq!(info.size_bytes, 52_428_800);
        assert_eq!(info.bitrate, Some(20_930_000));
        assert!(info.has_audio);
    }

    #[test]
    fn real_file_size_wins_over_ffprobe() {
        let info = parse(SAMPLE, "C:/v/clip.mp4", Some(123)).unwrap();
        assert_eq!(info.size_bytes, 123);
    }

    #[test]
    fn rejects_audio_only() {
        let json = r#"{"streams":[{"codec_type":"audio","codec_name":"mp3"}],"format":{"duration":"3.0"}}"#;
        assert!(matches!(parse(json, "a.mp3", None), Err(AppError::InvalidInput(InputProblem::NoVideoStream))));
    }

    #[test]
    fn webm_duration_comes_from_stream_tag() {
        let json = r#"{
          "streams": [
            { "codec_type": "video", "codec_name": "vp9", "width": 1280, "height": 720,
              "tags": { "DURATION": "00:01:23.456000000" } }
          ],
          "format": { "format_name": "matroska,webm" }
        }"#;
        let info = parse(json, "C:/v/rec.webm", Some(1)).unwrap();
        assert!((info.duration_s - 83.456).abs() < 1e-9);
    }

    #[test]
    fn missing_duration_is_left_for_scan() {
        let json = r#"{"streams":[{"codec_type":"video","width":640,"height":480}],"format":{}}"#;
        assert_eq!(parse(json, "C:/v/rec.webm", Some(1)).unwrap().duration_s, 0.0);
    }

    #[test]
    fn detects_hdr_transfer() {
        let json = r#"{"streams":[{"codec_type":"video","width":3840,"height":2160,
            "color_transfer":"arib-std-b67"}],"format":{"duration":"5.0"}}"#;
        assert!(parse(json, "C:/v/iphone.mov", Some(1)).unwrap().hdr);
        assert!(!parse(SAMPLE, "C:/v/clip.mp4", None).unwrap().hdr);
    }

    #[test]
    fn parses_clock_values() {
        assert_eq!(parse_clock("00:01:23.5"), Some(83.5));
        assert_eq!(parse_clock("1:00:00"), Some(3600.0));
        assert_eq!(parse_clock("42.25"), Some(42.25));
        assert_eq!(parse_clock("abc"), None);
        assert_eq!(parse_clock("1:2:3:4"), None);
    }
}
