use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};

pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // High contrast Catppuccin Mocha / Slate Dark Palette
    visuals.override_text_color = Some(Color32::from_rgb(245, 247, 255)); // Crisp bright white base text
    visuals.window_fill = Color32::from_rgb(24, 24, 37);                 // Dark background
    visuals.panel_fill = Color32::from_rgb(30, 30, 46);                  // Panel background
    visuals.extreme_bg_color = Color32::from_rgb(17, 17, 27);            // Inputs background

    // Non-interactive widgets (labels, groups, backgrounds)
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(245, 247, 255));
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 46);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(58, 60, 80));

    // Widgets inactive (buttons, toggles)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(49, 50, 68);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(245, 247, 255));
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    // Widgets hovered
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(69, 71, 90);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.2_f32, Color32::from_rgb(255, 255, 255));
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    // Widgets active / pressed
    visuals.widgets.active.bg_fill = Color32::from_rgb(137, 180, 250); // Accent blue
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(17, 17, 27));
    visuals.widgets.active.rounding = Rounding::same(6.0);

    // Selection
    visuals.selection.bg_fill = Color32::from_rgb(137, 180, 250);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(245, 247, 255));

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(137, 180, 250);
pub const SUCCESS_GREEN: Color32 = Color32::from_rgb(166, 227, 161);
pub const ERROR_RED: Color32 = Color32::from_rgb(243, 139, 168);
pub const CARD_BG: Color32 = Color32::from_rgb(30, 30, 46);
pub const CARD_HOVER_BG: Color32 = Color32::from_rgb(49, 50, 68);
pub const CARD_SELECTED_BG: Color32 = Color32::from_rgb(69, 71, 90);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(250, 250, 255);   // Pure crisp white
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(186, 194, 222); // High legibility light slate
#[allow(dead_code)]
pub const TEXT_MUTED: Color32 = Color32::from_rgb(147, 153, 178);     // Subtle meta
