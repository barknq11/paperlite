use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct WallpaperItem {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub modified: SystemTime,
    pub dimensions: Option<(u32, u32)>,
}

impl WallpaperItem {
    pub fn formatted_size(&self) -> String {
        let kb = self.size_bytes as f64 / 1024.0;
        if kb < 1024.0 {
            format!("{:.1} KB", kb)
        } else {
            let mb = kb / 1024.0;
            format!("{:.2} MB", mb)
        }
    }

    pub fn formatted_dimensions(&self) -> String {
        match self.dimensions {
            Some((w, h)) => {
                let tag = if w >= 3840 && h >= 2160 {
                    " (4K)"
                } else if w >= 2560 && h >= 1440 {
                    " (2K)"
                } else if w >= 1920 && h >= 1080 {
                    " (FHD)"
                } else if (w as f64 / h as f64) > 2.0 {
                    " (UW)"
                } else {
                    ""
                };
                format!("{}x{}{}", w, h, tag)
            }
            None => "Loading...".to_string(),
        }
    }

    pub fn aspect_ratio_str(&self) -> String {
        if let Some((w, h)) = self.dimensions {
            if h == 0 { return "Unknown".to_string(); }
            let ratio = w as f64 / h as f64;
            if (ratio - 16.0 / 9.0).abs() < 0.05 {
                "16:9".to_string()
            } else if (ratio - 16.0 / 10.0).abs() < 0.05 {
                "16:10".to_string()
            } else if (ratio - 21.0 / 9.0).abs() < 0.08 {
                "21:9 Ultrawide".to_string()
            } else if (ratio - 32.0 / 9.0).abs() < 0.08 {
                "32:9 Super Ultrawide".to_string()
            } else if (ratio - 4.0 / 3.0).abs() < 0.05 {
                "4:3".to_string()
            } else if (ratio - 1.0).abs() < 0.05 {
                "1:1 Square".to_string()
            } else if ratio < 1.0 {
                "Portrait".to_string()
            } else {
                format!("{:.2}:1", ratio)
            }
        } else {
            "Unknown".to_string()
        }
    }
}

pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "bmp"
        )
    } else {
        false
    }
}

pub fn scan_directory(dir: &Path, recursive: bool) -> Vec<WallpaperItem> {
    let mut items = Vec::new();
    let max_depth = if recursive { 10 } else { 1 };

    for entry in WalkDir::new(dir)
        .max_depth(max_depth)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_image_file(path) {
            let metadata = entry.metadata().ok();
            let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata.as_ref().and_then(|m| m.modified().ok()).unwrap_or(SystemTime::UNIX_EPOCH);
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Fast header read for dimensions if possible
            let dimensions = image::image_dimensions(path).ok();

            items.push(WallpaperItem {
                path: path.to_path_buf(),
                name,
                size_bytes,
                modified,
                dimensions,
            });
        }
    }

    items
}
