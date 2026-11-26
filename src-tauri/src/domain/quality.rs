use super::model::Encoder;

fn range(encoder: Encoder) -> (u8, u8) {
    match encoder {
        Encoder::X264 => (18, 32),
        Encoder::X265 => (20, 34),
        Encoder::NvencH264 => (19, 33),
        Encoder::NvencHevc => (21, 35),
    }
}

pub fn quality_param(encoder: Encoder, quality: u8) -> u8 {
    let (best, worst) = range(encoder);
    let q = f32::from(quality.min(100)) / 100.0;
    let value = f32::from(worst) - f32::from(worst - best) * q;
    value.round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        assert_eq!(quality_param(Encoder::X264, 100), 18);
        assert_eq!(quality_param(Encoder::X264, 0), 32);
        assert_eq!(quality_param(Encoder::NvencHevc, 100), 21);
        assert_eq!(quality_param(Encoder::NvencHevc, 0), 35);
    }

    #[test]
    fn presets_land_inside_range() {
        assert_eq!(quality_param(Encoder::X264, 85), 20);
        assert_eq!(quality_param(Encoder::X264, 60), 24);
        assert_eq!(quality_param(Encoder::X264, 35), 27);
    }

    #[test]
    fn clamps_above_100() {
        assert_eq!(quality_param(Encoder::X265, 250), 20);
    }

    #[test]
    fn monotonic() {
        for enc in [Encoder::X264, Encoder::X265, Encoder::NvencH264, Encoder::NvencHevc] {
            let mut prev = u8::MAX;
            for q in 0..=100 {
                let v = quality_param(enc, q);
                assert!(v <= prev, "{enc:?} q={q}");
                prev = v;
            }
        }
    }
}
