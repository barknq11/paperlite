use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WallpaperMode {
    Fill,
    Fit,
    Stretch,
    Center,
    Tile,
}

impl WallpaperMode {
    pub const ALL: [WallpaperMode; 5] = [
        WallpaperMode::Fill,
        WallpaperMode::Fit,
        WallpaperMode::Stretch,
        WallpaperMode::Center,
        WallpaperMode::Tile,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            WallpaperMode::Fill => "Fill",
            WallpaperMode::Fit => "Fit",
            WallpaperMode::Stretch => "Stretch",
            WallpaperMode::Center => "Center",
            WallpaperMode::Tile => "Tile",
        }
    }

    #[allow(dead_code)]
    pub fn to_swaybg_mode(&self) -> &'static str {
        match self {
            WallpaperMode::Fill => "fill",
            WallpaperMode::Fit => "fit",
            WallpaperMode::Stretch => "stretch",
            WallpaperMode::Center => "center",
            WallpaperMode::Tile => "tile",
        }
    }
}

pub fn apply_wallpaper(path: &Path, mode: WallpaperMode) -> Result<String, String> {
    #[cfg(windows)]
    {
        windows::set_wallpaper(path, mode)
    }

    #[cfg(target_os = "linux")]
    {
        linux::set_wallpaper(path, mode)
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

#[cfg(windows)]
pub mod windows;

#[cfg(any(target_os = "linux", not(windows)))]
pub mod linux;
