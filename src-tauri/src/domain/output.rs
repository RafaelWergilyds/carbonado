use std::path::{Path, PathBuf};

const SUFFIX: &str = "_carbonado";

pub fn suggest(input: &Path, dir: Option<&Path>, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let dir = dir
        .or_else(|| input.parent())
        .unwrap_or_else(|| Path::new(""));
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "video".into());

    let candidate = dir.join(format!("{stem}{SUFFIX}.mp4"));
    if !exists(&candidate) {
        return candidate;
    }
    (2u32..)
        .map(|n| dir.join(format!("{stem}{SUFFIX} ({n}).mp4")))
        .find(|p| !exists(p))
        .expect("intervalo infinito sempre encontra um nome livre")
}

pub fn normalize(output: &Path) -> PathBuf {
    let is_mp4 = output
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("mp4"));
    if is_mp4 {
        output.to_path_buf()
    } else {
        let mut name = output.file_name().unwrap_or_default().to_os_string();
        name.push(".mp4");
        output.with_file_name(name)
    }
}

pub fn same_file(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/").to_lowercase();
    norm(a) == norm(b)
}

pub fn temp_path(output: &Path) -> PathBuf {
    let stem = output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "video".into());
    output.with_file_name(format!("{stem}.carbonado-part.mp4"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn suggests_next_to_input() {
        let p = suggest(Path::new("C:/v/clip.mov"), None, |_| false);
        assert_eq!(p, PathBuf::from("C:/v/clip_carbonado.mp4"));
    }

    #[test]
    fn suggests_in_chosen_dir() {
        let p = suggest(Path::new("C:/v/clip.mov"), Some(Path::new("D:/out")), |_| false);
        assert_eq!(p, PathBuf::from("D:/out/clip_carbonado.mp4"));
    }

    #[test]
    fn avoids_collisions() {
        let taken: HashSet<PathBuf> = [
            PathBuf::from("C:/v/clip_carbonado.mp4"),
            PathBuf::from("C:/v/clip_carbonado (2).mp4"),
        ]
        .into();
        let p = suggest(Path::new("C:/v/clip.mp4"), None, |p| taken.contains(p));
        assert_eq!(p, PathBuf::from("C:/v/clip_carbonado (3).mp4"));
    }

    #[test]
    fn normalize_forces_mp4() {
        assert_eq!(normalize(Path::new("C:/o/a.MP4")), PathBuf::from("C:/o/a.MP4"));
        assert_eq!(normalize(Path::new("C:/o/a")), PathBuf::from("C:/o/a.mp4"));
        assert_eq!(normalize(Path::new("C:/o/a.mov")), PathBuf::from("C:/o/a.mov.mp4"));
    }

    #[test]
    fn same_file_ignores_case_and_separators() {
        assert!(same_file(Path::new("C:\\V\\Clip.mp4"), Path::new("c:/v/clip.mp4")));
        assert!(!same_file(Path::new("C:/v/a.mp4"), Path::new("C:/v/b.mp4")));
    }

    #[test]
    fn temp_is_beside_output() {
        assert_eq!(
            temp_path(Path::new("D:/out/clip.mp4")),
            PathBuf::from("D:/out/clip.carbonado-part.mp4")
        );
    }
}
