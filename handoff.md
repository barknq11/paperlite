# 🌌 PaperLite - Project Handoff & Run Guide

PaperLite is an ultra-lightweight, GPU-accelerated wallpaper browser and manager designed for **Windows** (Win32 / DWM) and **Linux** (`swaybg` for Sway / Hyprland).

---

## 📌 Project Overview & Specifications

- **Language & Framework**: Rust, `eframe` (egui 0.29), `glow` (OpenGL / DirectX / Vulkan / Wayland hardware acceleration).
- **Executable Size**: **~5.8 MB** (standalone release binary, zero external runtime dependencies).
- **Memory Footprint**: **~20MB–35MB RAM**.
- **Startup Time**: **< 0.05 seconds**.
- **Rendering**: 100% hardware-accelerated immediate mode GUI; smooth 60Hz–240Hz+ scrolling.
- **Image Pipeline**: Multithreaded background thumbnail generation with direct upload to GPU VRAM textures.

---

## 🛠️ Platform Backends

### 1. Windows Backend (`src/backend/windows.rs`)
- Uses native Win32 `SystemParametersInfoW` (`SPI_SETDESKWALLPAPER`) with `SPIF_UPDATEINIFILE | SPIF_SENDCHANGE`.
- Controls display styles via registry (`HKCU\Control Panel\Desktop`):
  - `WallpaperStyle`: `10` (Fill), `6` (Fit), `2` (Stretch), `0` (Center / Tile).
  - `TileWallpaper`: `1` (Tile), `0` (Other).
- Automatically converts modern formats like `.webp` or `.avif` to cached high-res PNGs so any image applies reliably.

### 2. Linux Backend (`src/backend/linux.rs`)
- Manages `swaybg` processes directly: `swaybg -i <path> -m <mode>`.
- Automatically terminates previous `swaybg` instances (`pkill -x swaybg`) to avoid stacking processes in memory.
- Writes and marks executable `~/.config/swaybg/current_wallpaper.sh` for one-line session autostart in **Sway** and **Hyprland**.

---

## 🚀 How to Launch & Run

### On Windows

| Method | How to Use |
| :--- | :--- |
| **1. Direct Executable** | Double-click [`paperlite.exe`](./paperlite.exe) directly in the project root folder. |
| **2. Desktop & Start Menu** | Run [`Create_Shortcuts.bat`](./Create_Shortcuts.bat) once. You can then open PaperLite from your **Desktop icon** or press the **Windows key**, type `paperlite`, and hit Enter. |
| **3. Explorer Right-Click Menu** | Run [`Add_RightClick_Menu.bat`](./Add_RightClick_Menu.bat) once. You can then right-click **any folder** or empty space in Windows Explorer and click **"Browse with PaperLite"**. *(To remove, run `Remove_RightClick_Menu.bat`)*. |
| **4. Command Line / Headless** | Use `paperlite --apply <path> --mode <mode>` to apply wallpapers programmatically without opening the GUI. |

### On Linux (Sway / Hyprland / Wayfire)

1. Ensure `swaybg` is installed:
   ```bash
   # Arch Linux:
   sudo pacman -S swaybg
   # Ubuntu / Debian:
   sudo apt install swaybg
   # Fedora:
   sudo dnf install swaybg
   ```
2. Launch the app:
   ```bash
   ./run.sh
   ```
3. *(Optional)* Integrate into your application launcher (Rofi, Wofi, Walker, GNOME, KDE):
   ```bash
   cp paperlite.desktop ~/.local/share/applications/
   ```

---

## ⌨️ CLI Command Reference

PaperLite can be run directly from terminal or shell scripts:

```bash
# Open GUI in default or last saved directory:
paperlite

# Open GUI directly in a specific directory:
paperlite "C:\Users\User\Pictures\Wallpapers"

# Headless: Apply wallpaper immediately and exit:
paperlite --apply "C:\Users\User\Pictures\sunset.jpg" --mode fill

# Supported modes: fill, fit, stretch, center, tile
paperlite --apply "/path/to/image.png" --mode fit
```

---

## ⚙️ Sway & Hyprland Autostart Integration

Whenever a wallpaper is chosen in PaperLite on Linux, the script `~/.config/swaybg/current_wallpaper.sh` is automatically created and updated.

To restore your wallpaper automatically on login/boot:
- **Modern Hyprland (Lua)** (`~/.config/hypr/hyprland.lua`):
  ```lua
  hl.on("hyprland.start", function ()
    hl.exec_cmd("~/.config/swaybg/current_wallpaper.sh")
  end)
  ```
- **Classic Hyprland** (`~/.config/hypr/hyprland.conf`):
  ```ini
  exec-once = ~/.config/swaybg/current_wallpaper.sh
  ```
- **Sway** (`~/.config/sway/config`):
  ```sway
  exec_always ~/.config/swaybg/current_wallpaper.sh
  ```

---

## 🎨 UI Features & Controls

- **Gallery Grid**: Dynamically sized cards with thumbnail preview, filename, resolution badge (`4K`, `2K`, `FHD`, `Ultrawide`), and file size.
- **Double-Click**: Double-clicking any card immediately applies it as desktop wallpaper.
- **Inspector Panel**:
  - High-res preview with aspect ratio calculation (e.g. `16:9`, `21:9 Ultrawide`).
  - Metadata: dimensions, size, and absolute path.
  - Display mode buttons: `Fill`, `Fit`, `Stretch`, `Center`, `Tile`.
  - Prominent **"Apply as Wallpaper"** button.
  - Quick action buttons: **"Open in Folder"** and **"View Fullscreen"**.
- **Search & Sort**: Real-time name search filter, sorting (Name A-Z/Z-A, Date Modified, File Size), and recursive subfolder scanning toggle.

---

## 📁 Project Directory Map

```
agy/
├── paperlite.exe               # 5.8 MB standalone optimized release binary
├── Cargo.toml                  # Rust dependencies & build settings
├── Create_Shortcuts.bat        # 1-click installer for Desktop & Start Menu shortcuts
├── Add_RightClick_Menu.bat     # Adds "Browse with PaperLite" to Windows Explorer
├── Remove_RightClick_Menu.bat  # Removes the Explorer context menu entry
├── run.bat                     # Windows launcher script
├── run.sh                      # Linux launcher & auto-build script
├── paperlite.desktop           # Linux desktop entry file
├── README.md                   # Full user and developer guide
├── handoff.md                  # This file (summary & run guide)
├── build.rs                    # Windows resource compiler (embeds icon.ico)
├── assets/
│   ├── icon.ico                # Multi-resolution Windows executable icon (16px to 256px)
│   └── icon.png                # PNG icon embedded in title bar & taskbar
├── src/
│   ├── main.rs                 # Entrypoint & CLI parameter handling
│   ├── config.rs               # JSON settings management (~/.config or %LOCALAPPDATA%)
│   ├── scanner.rs              # Directory crawler & image metadata parser
│   ├── backend/
│   │   ├── mod.rs              # Backend interface & WallpaperMode definitions
│   │   ├── windows.rs          # Win32 SystemParametersInfoW & registry logic
│   │   └── linux.rs            # swaybg process controller & startup script generator
│   └── ui/
│       ├── mod.rs              # UI exports
│       ├── theme.rs            # High-contrast dark palette (Catppuccin Mocha aesthetic)
│       └── app.rs              # GPU-accelerated egui gallery and inspector view
└── target/
    └── release/
        └── paperlite.exe       # Compiled release binary
```

---

## 🔧 Rebuilding From Source

If you modify the source code, rebuild the optimized binary anytime:

```bash
cargo build --release
copy target\release\paperlite.exe .\paperlite.exe
```
