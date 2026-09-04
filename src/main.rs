#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use eframe::egui;
use image::GenericImageView;

mod backend;
mod config;
mod scanner;
mod ui;

use backend::{apply_wallpaper, WallpaperMode};
use ui::WallpaperApp;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut target_path: Option<PathBuf> = None;

    // CLI mode check
    if args.len() > 1 {
        if args[1] == "--help" || args[1] == "-h" {
            #[cfg(windows)]
            unsafe {
                windows_sys::Win32::System::Console::AttachConsole(u32::MAX);
            }
            print_help();
            return Ok(());
        }

        if args[1] == "--apply" || args[1] == "-a" {
            #[cfg(windows)]
            unsafe {
                windows_sys::Win32::System::Console::AttachConsole(u32::MAX);
            }

            if args.len() < 3 {
                eprintln!("Error: --apply requires an image path argument.");
                std::process::exit(1);
            }

            let path = PathBuf::from(&args[2]);
            let mut mode = WallpaperMode::Fill;

            if args.len() >= 5 && (args[3] == "--mode" || args[3] == "-m") {
                mode = match args[4].to_lowercase().as_str() {
                    "fit" => WallpaperMode::Fit,
                    "stretch" => WallpaperMode::Stretch,
                    "center" => WallpaperMode::Center,
                    "tile" => WallpaperMode::Tile,
                    _ => WallpaperMode::Fill,
                };
            }

            match apply_wallpaper(&path, mode) {
                Ok(msg) => {
                    println!("{}", msg);
                    return Ok(());
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }

        // If user passed a folder or file directly: e.g. `paperlite "C:\Wallpapers"`
        let candidate = PathBuf::from(&args[1]);
        if candidate.exists() {
            target_path = Some(candidate);
        }
    }

    // Load embedded window & taskbar icon
    let icon = load_app_icon();

    // GUI mode
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("PaperLite - Wallpaper Manager")
        .with_inner_size([1150.0, 720.0])
        .with_min_inner_size([760.0, 520.0])
        .with_drag_and_drop(true);

    if let Some(ic) = icon {
        viewport = viewport.with_icon(ic);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "PaperLite",
        native_options,
        Box::new(move |cc| Ok(Box::new(WallpaperApp::new(cc, target_path)))),
    )
}

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        return Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        });
    }
    None
}

fn print_help() {
    println!(r#"
PaperLite - Lightweight GPU-Accelerated Wallpaper Manager
Supports Windows (Win32 / DWM) and Linux (swaybg)

USAGE:
    paperlite [OPTIONS] [DIRECTORY_OR_FILE]

OPTIONS:
    -h, --help                  Print help information
    -a, --apply <FILE>          Apply the specified image file as desktop wallpaper directly
    -m, --mode <MODE>           Wallpaper display mode: fill, fit, stretch, center, tile (default: fill)

EXAMPLES:
    # Open GPU-accelerated graphical browser:
    paperlite

    # Open specific folder directly:
    paperlite C:\Users\User\Pictures\Wallpapers

    # Apply wallpaper from terminal or script:
    paperlite --apply ~/Pictures/sunset.jpg --mode fill
"#);
}
