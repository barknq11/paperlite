use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{self, Color32, Rounding, Stroke, TextureHandle, Vec2};

use crate::backend::{apply_wallpaper, WallpaperMode};
use crate::config::AppConfig;
use crate::scanner::{scan_directory, WallpaperItem};
use crate::ui::theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    NameAsc,
    NameDesc,
    DateNewest,
    DateOldest,
    SizeLargest,
}

impl SortOption {
    pub const ALL: [SortOption; 5] = [
        SortOption::NameAsc,
        SortOption::NameDesc,
        SortOption::DateNewest,
        SortOption::DateOldest,
        SortOption::SizeLargest,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SortOption::NameAsc => "Name (A-Z)",
            SortOption::NameDesc => "Name (Z-A)",
            SortOption::DateNewest => "Date (Newest)",
            SortOption::DateOldest => "Date (Oldest)",
            SortOption::SizeLargest => "Size (Largest)",
        }
    }
}

pub struct WallpaperApp {
    config: AppConfig,
    current_dir: Option<PathBuf>,
    items: Vec<WallpaperItem>,
    filtered_indices: Vec<usize>,
    selected_index: Option<usize>,
    selected_mode: WallpaperMode,
    search_query: String,
    sort_by: SortOption,
    status_message: Option<(String, bool, Instant)>,

    // Async thumbnails to GPU
    thumb_tx: Sender<(PathBuf, egui::ColorImage)>,
    thumb_rx: Receiver<(PathBuf, egui::ColorImage)>,
    textures: HashMap<PathBuf, TextureHandle>,
    loading_paths: HashSet<PathBuf>,

    // Preview texture
    preview_tx: Sender<(PathBuf, egui::ColorImage)>,
    preview_rx: Receiver<(PathBuf, egui::ColorImage)>,
    preview_texture: Option<(PathBuf, TextureHandle)>,
    loading_preview: Option<PathBuf>,
}

impl WallpaperApp {
    pub fn new(cc: &eframe::CreationContext<'_>, target_path: Option<PathBuf>) -> Self {
        apply_dark_theme(&cc.egui_ctx);

        let config = AppConfig::load();
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (preview_tx, preview_rx) = crossbeam_channel::unbounded();

        let (initial_dir, target_file) = match target_path {
            Some(ref p) if p.is_dir() => (Some(p.clone()), None),
            Some(ref p) if p.is_file() => (p.parent().map(|d| d.to_path_buf()), Some(p.clone())),
            _ => (config.last_folder.clone(), None),
        };

        let mode = config.default_mode;

        let mut app = Self {
            config,
            current_dir: initial_dir.clone(),
            items: Vec::new(),
            filtered_indices: Vec::new(),
            selected_index: None,
            selected_mode: mode,
            search_query: String::new(),
            sort_by: SortOption::NameAsc,
            status_message: None,

            thumb_tx,
            thumb_rx,
            textures: HashMap::new(),
            loading_paths: HashSet::new(),

            preview_tx,
            preview_rx,
            preview_texture: None,
            loading_preview: None,
        };

        if let Some(dir) = initial_dir {
            app.load_folder(dir);

            if let Some(file) = target_file {
                if let Some(pos) = app.items.iter().position(|it| it.path == file) {
                    if let Some(filtered_pos) = app.filtered_indices.iter().position(|&idx| idx == pos) {
                        app.select_item(filtered_pos);
                    }
                }
            }
        }

        app
    }

    pub fn load_folder(&mut self, dir: PathBuf) {
        self.current_dir = Some(dir.clone());
        self.config.add_recent(dir.clone());

        self.items = scan_directory(&dir, self.config.recursive_scan);
        self.textures.clear();
        self.loading_paths.clear();
        self.preview_texture = None;
        self.loading_preview = None;
        self.selected_index = None;

        self.apply_sort_and_filter();

        if !self.filtered_indices.is_empty() {
            self.select_item(0);
        }
    }

    pub fn apply_sort_and_filter(&mut self) {
        let query = self.search_query.trim().to_lowercase();

        let mut indices: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if query.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&query)
                }
            })
            .map(|(idx, _)| idx)
            .collect();

        match self.sort_by {
            SortOption::NameAsc => {
                indices.sort_by(|&a, &b| self.items[a].name.to_lowercase().cmp(&self.items[b].name.to_lowercase()));
            }
            SortOption::NameDesc => {
                indices.sort_by(|&a, &b| self.items[b].name.to_lowercase().cmp(&self.items[a].name.to_lowercase()));
            }
            SortOption::DateNewest => {
                indices.sort_by(|&a, &b| self.items[b].modified.cmp(&self.items[a].modified));
            }
            SortOption::DateOldest => {
                indices.sort_by(|&a, &b| self.items[a].modified.cmp(&self.items[b].modified));
            }
            SortOption::SizeLargest => {
                indices.sort_by(|&a, &b| self.items[b].size_bytes.cmp(&self.items[a].size_bytes));
            }
        }

        self.filtered_indices = indices;

        // Keep selection valid
        if let Some(sel) = self.selected_index {
            if !self.filtered_indices.contains(&sel) {
                self.selected_index = self.filtered_indices.first().copied();
                if let Some(new_sel) = self.selected_index {
                    let path = self.items[new_sel].path.clone();
                    self.request_preview(path);
                }
            }
        }
    }

    fn select_item(&mut self, filtered_pos: usize) {
        if let Some(&item_idx) = self.filtered_indices.get(filtered_pos) {
            self.selected_index = Some(item_idx);
            let path = self.items[item_idx].path.clone();
            self.request_preview(path);
        }
    }

    fn request_thumbnail(&mut self, path: PathBuf) {
        if self.textures.contains_key(&path) || self.loading_paths.contains(&path) {
            return;
        }

        self.loading_paths.insert(path.clone());
        let tx = self.thumb_tx.clone();

        std::thread::spawn(move || {
            if let Ok(img) = image::open(&path) {
                let thumb = img.thumbnail(320, 200);
                let size = [thumb.width() as usize, thumb.height() as usize];
                let rgba = thumb.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let _ = tx.send((path, color_image));
            }
        });
    }

    fn request_preview(&mut self, path: PathBuf) {
        if let Some((curr_p, _)) = &self.preview_texture {
            if curr_p == &path {
                return;
            }
        }

        self.loading_preview = Some(path.clone());
        let tx = self.preview_tx.clone();

        std::thread::spawn(move || {
            if let Ok(img) = image::open(&path) {
                let preview = img.thumbnail(960, 600);
                let size = [preview.width() as usize, preview.height() as usize];
                let rgba = preview.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let _ = tx.send((path, color_image));
            }
        });
    }

    fn apply_current_wallpaper(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(item) = self.items.get(idx) {
                match apply_wallpaper(&item.path, self.selected_mode) {
                    Ok(msg) => {
                        self.status_message = Some((msg, false, Instant::now()));
                    }
                    Err(err) => {
                        self.status_message = Some((err, true, Instant::now()));
                    }
                }
            }
        }
    }

    fn open_in_folder(&self, path: &Path) {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("explorer")
                .arg(format!("/select,\"{}\"", path.display()))
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let dir = path.parent().unwrap_or(path);
            let _ = std::process::Command::new("xdg-open")
                .arg(dir)
                .spawn();
        }
    }

    fn open_default_viewer(&self, path: &Path) {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", "", &path.to_string_lossy()])
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn();
        }
    }
}

impl eframe::App for WallpaperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain thumbnail channel into GPU textures
        let mut got_thumbs = false;
        while let Ok((path, color_image)) = self.thumb_rx.try_recv() {
            let handle = ctx.load_texture(
                format!("thumb_{}", path.display()),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.textures.insert(path, handle);
            got_thumbs = true;
        }

        // Drain preview channel
        while let Ok((path, color_image)) = self.preview_rx.try_recv() {
            let handle = ctx.load_texture(
                format!("preview_{}", path.display()),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.preview_texture = Some((path, handle));
            got_thumbs = true;
        }

        if got_thumbs {
            ctx.request_repaint();
        }

        // Top Panel: Header & Controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("PaperLite").color(TEXT_PRIMARY).strong());
                ui.label(
                    egui::RichText::new("• GPU Accelerated Wallpaper Manager")
                        .color(TEXT_SECONDARY)
                        .size(12.0),
                );

                #[cfg(target_os = "linux")]
                ui.label(
                    egui::RichText::new("[Backend: swaybg]")
                        .color(ACCENT_BLUE)
                        .size(11.0),
                );

                #[cfg(windows)]
                ui.label(
                    egui::RichText::new("[Backend: Windows Win32 / DWM]")
                        .color(ACCENT_BLUE)
                        .size(11.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Refresh").color(TEXT_PRIMARY)).clicked() {
                        if let Some(dir) = self.current_dir.clone() {
                            self.load_folder(dir);
                        }
                    }

                    if ui.button(egui::RichText::new("Browse Folder...").color(TEXT_PRIMARY)).clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.load_folder(folder);
                        }
                    }

                    if let Some(dir) = &self.current_dir {
                        let path_str = dir.display().to_string();
                        let display_str = if path_str.len() > 40 {
                            format!("...{}", &path_str[path_str.len().saturating_sub(37)..])
                        } else {
                            path_str
                        };
                        ui.label(egui::RichText::new(display_str).color(TEXT_SECONDARY).monospace());
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Filter & Search bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Search:").color(TEXT_PRIMARY));
                let search_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Filter wallpapers by name...")
                        .text_color(TEXT_PRIMARY)
                        .desired_width(220.0),
                );
                if search_resp.changed() {
                    self.apply_sort_and_filter();
                }
                if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                    self.search_query.clear();
                    self.apply_sort_and_filter();
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Sort:").color(TEXT_PRIMARY));
                egui::ComboBox::from_id_salt("sort_combo")
                    .selected_text(egui::RichText::new(self.sort_by.as_str()).color(TEXT_PRIMARY))
                    .show_ui(ui, |ui| {
                        for opt in SortOption::ALL {
                            let text = egui::RichText::new(opt.as_str()).color(TEXT_PRIMARY);
                            if ui.selectable_value(&mut self.sort_by, opt, text).clicked() {
                                self.apply_sort_and_filter();
                            }
                        }
                    });

                ui.add_space(12.0);
                let rec_check = egui::Checkbox::new(
                    &mut self.config.recursive_scan,
                    egui::RichText::new("Include Subfolders").color(TEXT_PRIMARY),
                );
                if ui.add(rec_check).changed() {
                    self.config.save();
                    if let Some(dir) = self.current_dir.clone() {
                        self.load_folder(dir);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Showing {} of {} wallpapers",
                            self.filtered_indices.len(),
                            self.items.len()
                        ))
                        .color(TEXT_SECONDARY),
                    );
                });
            });

            ui.add_space(4.0);
        });

        // Bottom Status Toast
        if let Some((ref msg, is_err, time)) = self.status_message {
            let elapsed = time.elapsed();
            if elapsed < Duration::from_secs(4) {
                egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let color = if is_err { ERROR_RED } else { SUCCESS_GREEN };
                        let icon = if is_err { "[X]" } else { "[OK]" };
                        ui.label(egui::RichText::new(format!("{} {}", icon, msg)).color(color).strong());
                    });
                });
            } else {
                self.status_message = None;
            }
        }

        // Clone selected item details so right inspector doesn't hold reference to self.items
        let selected_item = self.selected_index.and_then(|idx| self.items.get(idx).cloned());
        let mut do_apply = false;
        let mut do_open_folder = None;
        let mut do_open_viewer = None;

        // Right Inspector Panel
        egui::SidePanel::right("inspector_panel")
            .min_width(320.0)
            .max_width(420.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading(egui::RichText::new("Wallpaper Details").color(TEXT_PRIMARY).strong());
                ui.add_space(8.0);

                if let Some(ref item) = selected_item {
                    // High-res preview
                    let preview_rect = ui.available_size_before_wrap();
                    let max_preview_h = 200.0f32.min(preview_rect.y * 0.4);

                    if let Some((_, ref handle)) = self.preview_texture {
                        let tex_size = handle.size_vec2();
                        let aspect = tex_size.x / tex_size.y.max(1.0);
                        let target_w = (max_preview_h * aspect).min(ui.available_width());
                        let target_h = target_w / aspect;

                        ui.vertical_centered(|ui| {
                            ui.image((handle.id(), Vec2::new(target_w, target_h)));
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 160.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, Rounding::same(8.0), CARD_BG);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Loading preview...",
                                egui::FontId::proportional(14.0),
                                TEXT_SECONDARY,
                            );
                        });
                    }

                    ui.add_space(10.0);
                    // Filename in bright, bold, crystal clear text
                    ui.label(
                        egui::RichText::new(&item.name)
                            .color(TEXT_PRIMARY)
                            .strong()
                            .size(16.0),
                    );

                    ui.add_space(6.0);
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Dimensions:").color(TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new(item.formatted_dimensions())
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Aspect Ratio:").color(TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new(item.aspect_ratio_str())
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("File Size:").color(TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new(item.formatted_size())
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Path:").color(TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new(item.path.display().to_string())
                                    .color(Color32::from_rgb(166, 227, 161))
                                    .size(11.0)
                                    .monospace(),
                            );
                        });
                    });

                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Display Mode:")
                            .color(TEXT_PRIMARY)
                            .strong()
                            .size(13.0),
                    );

                    ui.horizontal_wrapped(|ui| {
                        for mode in WallpaperMode::ALL {
                            let is_selected = self.selected_mode == mode;
                            let label_text = egui::RichText::new(mode.as_str())
                                .color(if is_selected {
                                    Color32::from_rgb(17, 17, 27)
                                } else {
                                    TEXT_PRIMARY
                                })
                                .strong();

                            if ui.selectable_label(is_selected, label_text).clicked() {
                                self.selected_mode = mode;
                                self.config.default_mode = self.selected_mode;
                                self.config.save();
                            }
                        }
                    });

                    ui.add_space(16.0);

                    // Prominent Apply Button
                    let apply_btn = egui::Button::new(
                        egui::RichText::new("Apply as Wallpaper")
                            .size(15.0)
                            .color(Color32::from_rgb(17, 17, 27))
                            .strong(),
                    )
                    .fill(ACCENT_BLUE)
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(ui.available_width(), 42.0));

                    if ui.add(apply_btn).clicked() {
                        do_apply = true;
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("Open in Folder").color(TEXT_PRIMARY)).clicked() {
                            do_open_folder = Some(item.path.clone());
                        }
                        if ui.button(egui::RichText::new("View Fullscreen").color(TEXT_PRIMARY)).clicked() {
                            do_open_viewer = Some(item.path.clone());
                        }
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new("Select a wallpaper from the gallery to preview")
                                .color(TEXT_SECONDARY),
                        );
                    });
                }
            });

        if do_apply {
            self.apply_current_wallpaper();
        }
        if let Some(path) = do_open_folder {
            self.open_in_folder(&path);
        }
        if let Some(path) = do_open_viewer {
            self.open_default_viewer(&path);
        }

        // Center Panel: Wallpaper Gallery Grid
        let mut need_thumbs: Vec<PathBuf> = Vec::new();
        let mut action_select: Option<usize> = None;
        let mut action_double_click = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.filtered_indices.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    if self.items.is_empty() {
                        ui.heading(egui::RichText::new("No wallpapers found").color(TEXT_PRIMARY));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Click 'Browse Folder...' in the top bar to select a folder containing wallpapers.")
                                .color(TEXT_SECONDARY),
                        );
                    } else {
                        ui.heading(egui::RichText::new("No matching wallpapers").color(TEXT_PRIMARY));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("No wallpapers matched '{}'", self.search_query))
                                .color(TEXT_SECONDARY),
                        );
                    }
                });
                return;
            }

            let card_w = 200.0f32;
            let card_h = 160.0f32;
            let spacing = 12.0f32;

            let avail_w = ui.available_width();
            let cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let total_items = self.filtered_indices.len();
                    let rows = (total_items + cols - 1) / cols;

                    for r in 0..rows {
                        ui.horizontal(|ui| {
                            for c in 0..cols {
                                let item_idx_pos = r * cols + c;
                                if item_idx_pos >= total_items {
                                    break;
                                }

                                let actual_item_idx = self.filtered_indices[item_idx_pos];
                                let item = &self.items[actual_item_idx];
                                let is_selected = self.selected_index == Some(actual_item_idx);

                                // Request GPU thumbnail queue
                                if !self.textures.contains_key(&item.path) && !self.loading_paths.contains(&item.path) {
                                    need_thumbs.push(item.path.clone());
                                }

                                // Card rendering
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(card_w, card_h),
                                    egui::Sense::click(),
                                );

                                if response.clicked() {
                                    action_select = Some(item_idx_pos);
                                }

                                if response.double_clicked() {
                                    action_select = Some(item_idx_pos);
                                    action_double_click = true;
                                }

                                // Card styling
                                let bg_color = if is_selected {
                                    CARD_SELECTED_BG
                                } else if response.hovered() {
                                    CARD_HOVER_BG
                                } else {
                                    CARD_BG
                                };

                                let border_stroke = if is_selected {
                                    Stroke::new(2.0_f32, ACCENT_BLUE)
                                } else if response.hovered() {
                                    Stroke::new(1.0_f32, Color32::from_rgb(108, 112, 134))
                                } else {
                                    Stroke::NONE
                                };

                                ui.painter().rect(rect, Rounding::same(8.0), bg_color, border_stroke);

                                // Draw thumbnail inside card
                                let thumb_h = 105.0f32;
                                let thumb_rect = egui::Rect::from_min_size(
                                    rect.min + Vec2::new(6.0, 6.0),
                                    Vec2::new(card_w - 12.0, thumb_h),
                                );

                                if let Some(tex) = self.textures.get(&item.path) {
                                    let tex_size = tex.size_vec2();
                                    let aspect = tex_size.x / tex_size.y.max(1.0);
                                    let mut draw_w = thumb_rect.width();
                                    let mut draw_h = draw_w / aspect;
                                    if draw_h > thumb_rect.height() {
                                        draw_h = thumb_rect.height();
                                        draw_w = draw_h * aspect;
                                    }

                                    let center = thumb_rect.center();
                                    let image_draw_rect = egui::Rect::from_center_size(center, Vec2::new(draw_w, draw_h));

                                    ui.painter().rect_filled(
                                        thumb_rect,
                                        Rounding::same(6.0),
                                        Color32::from_rgb(17, 17, 27),
                                    );
                                    ui.painter().image(
                                        tex.id(),
                                        image_draw_rect,
                                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                } else {
                                    ui.painter().rect_filled(
                                        thumb_rect,
                                        Rounding::same(6.0),
                                        Color32::from_rgb(17, 17, 27),
                                    );
                                    ui.painter().text(
                                        thumb_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "...",
                                        egui::FontId::proportional(14.0),
                                        TEXT_SECONDARY,
                                    );
                                }

                                // Filename text - pure bright white
                                let text_pos = rect.min + Vec2::new(8.0, thumb_h + 10.0);
                                let display_name = if item.name.len() > 22 {
                                    format!("{}...", &item.name[..19])
                                } else {
                                    item.name.clone()
                                };

                                ui.painter().text(
                                    text_pos,
                                    egui::Align2::LEFT_TOP,
                                    display_name,
                                    egui::FontId::proportional(12.5),
                                    TEXT_PRIMARY,
                                );

                                // Subtext: Resolution / Size badge - clearly visible light slate
                                let sub_pos = rect.min + Vec2::new(8.0, thumb_h + 28.0);
                                ui.painter().text(
                                    sub_pos,
                                    egui::Align2::LEFT_TOP,
                                    format!("{} • {}", item.formatted_dimensions(), item.formatted_size()),
                                    egui::FontId::proportional(11.0),
                                    TEXT_SECONDARY,
                                );

                                ui.add_space(spacing);
                            }
                        });
                        ui.add_space(spacing);
                    }
                });
        });

        // Request any thumbnails outside the loop
        for path in need_thumbs {
            self.request_thumbnail(path);
        }

        // Apply any selection outside the loop
        if let Some(pos) = action_select {
            self.select_item(pos);
            if action_double_click {
                self.apply_current_wallpaper();
            }
        }
    }
}
