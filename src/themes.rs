use bevy_egui::egui::{self, Color32};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    #[default]
    Matrix,
    Cyberpunk,
    Monochrome,
    Neon,
    Professional,
}

pub struct ThemeColors {
    pub primary: Color32,
    pub secondary: Color32,
    pub accent: Color32,
    pub background: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub border: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub note_colors: [Color32; 4], // For different instruments
}

impl ThemeColors {
    pub fn get(theme: Theme) -> Self {
        match theme {
            Theme::Matrix => ThemeColors {
                primary: Color32::from_rgb(0, 255, 0),
                secondary: Color32::from_rgb(0, 200, 0),
                accent: Color32::from_rgb(0, 150, 0),
                background: Color32::from_rgb(0, 0, 0),
                surface: Color32::from_rgb(5, 5, 5),
                text: Color32::from_rgb(0, 255, 0),
                text_dim: Color32::from_rgba_premultiplied(0, 150, 0, 150),
                border: Color32::from_rgb(0, 100, 0),
                hover: Color32::from_rgba_premultiplied(0, 50, 0, 100),
                active: Color32::from_rgba_premultiplied(0, 100, 0, 150),
                note_colors: [
                    Color32::from_rgb(0, 255, 0),      // SINE - Green
                    Color32::from_rgb(0, 150, 255),   // SQUARE - Blue
                    Color32::from_rgb(255, 150, 0),   // SAW - Orange
                    Color32::from_rgb(255, 0, 150),   // PULSE - Pink
                ],
            },
            Theme::Cyberpunk => ThemeColors {
                primary: Color32::from_rgb(255, 0, 255),
                secondary: Color32::from_rgb(0, 255, 255),
                accent: Color32::from_rgb(255, 255, 0),
                background: Color32::from_rgb(10, 5, 20),
                surface: Color32::from_rgb(20, 10, 30),
                text: Color32::from_rgb(255, 0, 255),
                text_dim: Color32::from_rgba_premultiplied(200, 0, 200, 150),
                border: Color32::from_rgb(100, 0, 100),
                hover: Color32::from_rgba_premultiplied(50, 0, 50, 100),
                active: Color32::from_rgba_premultiplied(100, 0, 100, 150),
                note_colors: [
                    Color32::from_rgb(255, 0, 255),   // SINE - Magenta
                    Color32::from_rgb(0, 255, 255),    // SQUARE - Cyan
                    Color32::from_rgb(255, 255, 0),    // SAW - Yellow
                    Color32::from_rgb(255, 100, 0),    // PULSE - Orange
                ],
            },
            Theme::Monochrome => ThemeColors {
                primary: Color32::from_rgb(255, 255, 255),
                secondary: Color32::from_rgb(200, 200, 200),
                accent: Color32::from_rgb(150, 150, 150),
                background: Color32::from_rgb(15, 15, 15),
                surface: Color32::from_rgb(25, 25, 25),
                text: Color32::from_rgb(255, 255, 255),
                text_dim: Color32::from_rgba_premultiplied(150, 150, 150, 200),
                border: Color32::from_rgb(80, 80, 80),
                hover: Color32::from_rgba_premultiplied(50, 50, 50, 100),
                active: Color32::from_rgba_premultiplied(100, 100, 100, 150),
                note_colors: [
                    Color32::from_rgb(255, 255, 255),  // SINE - White
                    Color32::from_rgb(200, 200, 200),  // SQUARE - Light Gray
                    Color32::from_rgb(150, 150, 150),   // SAW - Gray
                    Color32::from_rgb(100, 100, 100),  // PULSE - Dark Gray
                ],
            },
            Theme::Neon => ThemeColors {
                primary: Color32::from_rgb(0, 255, 255),
                secondary: Color32::from_rgb(255, 0, 255),
                accent: Color32::from_rgb(255, 255, 0),
                background: Color32::from_rgb(0, 0, 10),
                surface: Color32::from_rgb(5, 5, 15),
                text: Color32::from_rgb(0, 255, 255),
                text_dim: Color32::from_rgba_premultiplied(0, 200, 200, 150),
                border: Color32::from_rgb(0, 150, 150),
                hover: Color32::from_rgba_premultiplied(0, 50, 50, 100),
                active: Color32::from_rgba_premultiplied(0, 100, 100, 150),
                note_colors: [
                    Color32::from_rgb(0, 255, 255),     // SINE - Cyan
                    Color32::from_rgb(255, 0, 255),     // SQUARE - Magenta
                    Color32::from_rgb(255, 255, 0),     // SAW - Yellow
                    Color32::from_rgb(0, 255, 0),       // PULSE - Green
                ],
            },
            Theme::Professional => ThemeColors {
                primary: Color32::from_rgb(100, 150, 255),
                secondary: Color32::from_rgb(150, 200, 255),
                accent: Color32::from_rgb(255, 200, 100),
                background: Color32::from_rgb(20, 20, 25),
                surface: Color32::from_rgb(30, 30, 35),
                text: Color32::from_rgb(220, 220, 230),
                text_dim: Color32::from_rgba_premultiplied(150, 150, 160, 200),
                border: Color32::from_rgb(60, 80, 100),
                hover: Color32::from_rgba_premultiplied(40, 60, 80, 100),
                active: Color32::from_rgba_premultiplied(60, 100, 140, 150),
                note_colors: [
                    Color32::from_rgb(100, 200, 255),   // SINE - Light Blue
                    Color32::from_rgb(255, 150, 100),   // SQUARE - Orange
                    Color32::from_rgb(150, 255, 150),    // SAW - Light Green
                    Color32::from_rgb(255, 200, 100),   // PULSE - Yellow
                ],
            },
        }
    }

    pub fn apply_to_style(&self, style: &mut egui::Style) {
        style.visuals.widgets.noninteractive.bg_fill = self.background;
        style.visuals.window_fill = self.background;
        style.visuals.panel_fill = self.surface;
        style.visuals.selection.bg_fill = self.active;
        style.visuals.override_text_color = Some(self.text);
        style.visuals.widgets.inactive.bg_fill = self.surface;
        style.visuals.widgets.hovered.bg_fill = self.hover;
        style.visuals.widgets.active.bg_fill = self.active;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.border);
    }
}

