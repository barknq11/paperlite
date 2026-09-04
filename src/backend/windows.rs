use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

use super::WallpaperMode;

pub fn set_wallpaper(path: &Path, mode: WallpaperMode) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    // Windows native wallpaper API accepts JPG, PNG, BMP natively.
    // If the image is WebP or other modern formats, convert to a high-res PNG in cache.
    let ext_lower = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    let actual_path = match ext_lower.as_deref() {
        Some("webp") | Some("avif") => {
            let cache_dir = dirs_fallback().join("PaperLite");
            let _ = std::fs::create_dir_all(&cache_dir);
            let target_png = cache_dir.join("current_wallpaper.png");
            
            match image::open(path) {
                Ok(img) => {
                    if let Err(e) = img.save_with_format(&target_png, image::ImageFormat::Png) {
                        return Err(format!("Failed to convert image for Windows: {}", e));
                    }
                    target_png
                }
                Err(e) => return Err(format!("Failed to decode image: {}", e)),
            }
        }
        _ => path.to_path_buf(),
    };

    // Set registry style
    let (style_val, tile_val) = match mode {
        WallpaperMode::Fill => ("10", "0"),
        WallpaperMode::Fit => ("6", "0"),
        WallpaperMode::Stretch => ("2", "0"),
        WallpaperMode::Center => ("0", "0"),
        WallpaperMode::Tile => ("0", "1"),
    };

    let _ = Command::new("reg")
        .args(["add", r"HKCU\Control Panel\Desktop", "/v", "WallpaperStyle", "/t", "REG_SZ", "/d", style_val, "/f"])
        .output();

    let _ = Command::new("reg")
        .args(["add", r"HKCU\Control Panel\Desktop", "/v", "TileWallpaper", "/t", "REG_SZ", "/d", tile_val, "/f"])
        .output();

    // Call Win32 SystemParametersInfoW API
    let abs_path = if actual_path.is_absolute() {
        actual_path
    } else {
        std::env::current_dir().unwrap_or_default().join(&actual_path)
    };

    let wide_path: Vec<u16> = abs_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    let result = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide_path.as_ptr() as *mut _,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };

    if result != 0 {
        Ok(format!("Wallpaper set successfully on Windows ({})", mode.as_str()))
    } else {
        Err("Failed to set wallpaper via Windows SystemParametersInfoW API".to_string())
    }
}

fn dirs_fallback() -> std::path::PathBuf {
    if let Some(appdata) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(appdata)
    } else if let Some(temp) = std::env::var_os("TEMP") {
        std::path::PathBuf::from(temp)
    } else {
        std::path::PathBuf::from(".")
    }
}
