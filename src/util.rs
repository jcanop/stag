use std::path::{Path, PathBuf};

/// Ensure that the path has a PNG extensión.
///
/// Parameters:
/// * path: Path to validate.
///
/// Returns:
/// The path with the PNG extension.
pub fn ensure_png_extension(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    if path.extension().and_then(|ext| ext.to_str()) != Some("png") {
        path.set_extension("png");
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_png_extension_1() {
        let path = Path::new("images/picture.jpg");
        let png = ensure_png_extension(path);
        assert_eq!(Path::new("images/picture.png"), png);
    }

    #[test]
    fn test_ensure_png_extension_2() {
        let path = Path::new("images/picture.png");
        let png = ensure_png_extension(path);
        assert_eq!(Path::new("images/picture.png"), png);
    }

    #[test]
    fn test_ensure_png_extension_3() {
        let path = Path::new("images/picture");
        let png = ensure_png_extension(path);
        assert_eq!(Path::new("images/picture.png"), png);
    }
}
