use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressSnapshot {
    pub out_time_s: f64,
    pub speed: Option<f64>,
    pub finished: bool,
}

#[derive(Debug, Default)]
pub struct ProgressParser {
    out_time_us: Option<i64>,
    speed: Option<f64>,
}

impl ProgressParser {
    pub fn feed(&mut self, line: &str) -> Option<ProgressSnapshot> {
        let (key, value) = line.trim().split_once('=')?;
        let value = value.trim();
        match key {
            "out_time_us" | "out_time_ms" => {
                if let Ok(v) = value.parse::<i64>() {
                    self.out_time_us = Some(v.max(0));
                }
            }
            "speed" => {
                self.speed = value.trim_end_matches('x').trim().parse::<f64>().ok();
            }
            "progress" => {
                return Some(ProgressSnapshot {
                    out_time_s: self.out_time_us.unwrap_or(0) as f64 / 1_000_000.0,
                    speed: self.speed,
                    finished: value == "end",
                });
            }
            _ => {}
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub percent: f64,
    pub speed: Option<f64>,
    pub eta_s: Option<f64>,
}

impl ProgressEvent {
    pub fn from_snapshot(snap: ProgressSnapshot, duration_s: f64) -> Self {
        if snap.finished {
            return Self { percent: 100.0, speed: snap.speed, eta_s: Some(0.0) };
        }
        let percent = if duration_s > 0.0 {
            (snap.out_time_s / duration_s * 100.0).clamp(0.0, 99.9)
        } else {
            0.0
        };
        let eta_s = match snap.speed {
            Some(s) if s > 0.0 && duration_s > 0.0 => {
                Some(((duration_s - snap.out_time_s) / s).max(0.0))
            }
            _ => None,
        };
        Self { percent, speed: snap.speed, eta_s }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: &str = "frame=240
fps=120.00
stream_0_0_q=28.0
bitrate=1200.5kbits/s
total_size=1500000
out_time_us=10000000
out_time_ms=10000000
out_time=00:00:10.000000
dup_frames=0
drop_frames=0
speed=2.5x
progress=continue";

    #[test]
    fn parses_a_real_block() {
        let mut p = ProgressParser::default();
        let snaps: Vec<_> = BLOCK.lines().filter_map(|l| p.feed(l)).collect();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].out_time_s, 10.0);
        assert_eq!(snaps[0].speed, Some(2.5));
        assert!(!snaps[0].finished);

        let ev = ProgressEvent::from_snapshot(snaps[0], 20.0);
        assert_eq!(ev.percent, 50.0);
        assert_eq!(ev.eta_s, Some(4.0));
    }

    #[test]
    fn handles_na_values_at_start() {
        let mut p = ProgressParser::default();
        assert_eq!(p.feed("out_time_us=N/A"), None);
        assert_eq!(p.feed("speed=N/A"), None);
        let snap = p.feed("progress=continue").unwrap();
        assert_eq!(snap.out_time_s, 0.0);
        assert_eq!(snap.speed, None);
        assert_eq!(ProgressEvent::from_snapshot(snap, 30.0).eta_s, None);
    }

    #[test]
    fn end_reports_100() {
        let mut p = ProgressParser::default();
        p.feed("out_time_us=19900000");
        let snap = p.feed("progress=end").unwrap();
        assert!(snap.finished);
        assert_eq!(ProgressEvent::from_snapshot(snap, 20.0).percent, 100.0);
    }

    #[test]
    fn negative_time_is_clamped() {
        let mut p = ProgressParser::default();
        p.feed("out_time_us=-23220");
        let snap = p.feed("progress=continue").unwrap();
        assert_eq!(snap.out_time_s, 0.0);
    }
}
