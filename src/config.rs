use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::backend::WallpaperMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_folder: Option<PathBuf>,
    pub recent_folders: Vec<PathBuf>,
    pub default_mode: WallpaperMode,
    pub recursive_scan: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_folder: default_wallpaper_dir(),
            recent_folders: Vec::new(),
            default_mode: WallpaperMode::Fill,
            recursive_scan: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_file_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = config_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn add_recent(&mut self, folder: PathBuf) {
        self.recent_folders.retain(|f| f != &folder);
        self.recent_folders.insert(0, folder.clone());
        if self.recent_folders.len() > 8 {
            self.recent_folders.truncate(8);
        }
        self.last_folder = Some(folder);
        self.save();
    }
}

fn config_file_path() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(appdata).join("PaperLite").join("config.json");
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".config").join("paperlite").join("config.json");
        }
    }

    PathBuf::from("paperlite_config.json")
}

fn default_wallpaper_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(userprofile) = std::env::var_os("USERPROFILE") {
            let pics = PathBuf::from(userprofile).join("Pictures");
            if pics.exists() {
                return Some(pics);
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let pics = PathBuf::from(home).join("Pictures");
            if pics.exists() {
                return Some(pics);
            }
        }
    }

    None
}
