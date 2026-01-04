use ratatui::style::Color;

/// Enkai custom color palette
/// Gradient: Cyan → Blue → Purple → Magenta
pub struct EnkaiColors;

impl EnkaiColors {
    // Cyan shades (top/header elements)
    pub const CYAN_BRIGHT: Color = Color::Rgb(0, 255, 255);      // #00FFFF
    pub const CYAN_LIGHT: Color = Color::Rgb(64, 224, 255);      // #40E0FF

    // Blue shades (primary UI elements)
    pub const BLUE_MEDIUM: Color = Color::Rgb(0, 128, 255);      // #0080FF
    pub const BLUE_ROYAL: Color = Color::Rgb(65, 105, 225);      // #4169E1

    // Purple/Violet shades (secondary elements)
    pub const PURPLE_BRIGHT: Color = Color::Rgb(128, 0, 255);    // #8000FF
    pub const PURPLE_INDIGO: Color = Color::Rgb(75, 0, 130);     // #4B0082

    // Magenta/Pink shades (bottom/accent elements)
    pub const MAGENTA_BRIGHT: Color = Color::Rgb(255, 0, 255);   // #FF00FF
    pub const PINK_HOT: Color = Color::Rgb(255, 105, 180);       // #FF69B4

    // Semantic colors
    pub const SUCCESS: Color = Color::Rgb(0, 255, 200);          // Cyan-green
    pub const WARNING: Color = Color::Rgb(255, 200, 0);          // Yellow
    pub const ERROR: Color = Color::Rgb(255, 80, 120);           // Pink-red
    pub const INFO: Color = Color::Rgb(100, 180, 255);           // Light blue

    // UI state colors
    pub const FOCUS_BORDER: Color = Self::CYAN_BRIGHT;
    pub const INACTIVE_BORDER: Color = Color::Rgb(100, 100, 150);
    pub const SELECTED_BG: Color = Color::Rgb(40, 40, 80);
    pub const TEXT_NORMAL: Color = Color::Rgb(220, 220, 255);
    pub const TEXT_DIM: Color = Color::Rgb(120, 120, 160);
    pub const FOOTER_BG: Color = Color::Rgb(60, 60, 70);           // Lighter gray for footer

    // Code conflict colors
    pub const CONFLICT_CURRENT: Color = Self::CYAN_LIGHT;        // Current (HEAD)
    pub const CONFLICT_INCOMING: Color = Self::MAGENTA_BRIGHT;   // Incoming
    pub const CONFLICT_BOTH: Color = Self::PURPLE_BRIGHT;        // Both
    pub const CONFLICT_MARKER: Color = Self::BLUE_ROYAL;         // Markers (<<<, ===, >>>)

    // Conflict backgrounds (semi-transparent effect with darker tones)
    pub const CONFLICT_CURRENT_BG: Color = Color::Rgb(0, 60, 60);    // Dark cyan background
    pub const CONFLICT_INCOMING_BG: Color = Color::Rgb(60, 30, 0);   // Dark orange background

    // Resolved conflict background (light green)
    pub const RESOLVED_BG: Color = Color::Rgb(20, 60, 30);           // Subtle green background

    // Status indicators
    pub const STATUS_RESOLVED: Color = Self::SUCCESS;
    pub const STATUS_UNRESOLVED: Color = Self::ERROR;
}

/// Helper to create gradient text effects
pub fn gradient_char_color(index: usize, total: usize) -> Color {
    let ratio = index as f32 / total.max(1) as f32;

    // Gradient from cyan → magenta
    if ratio < 0.33 {
        // Cyan → Blue
        let local = ratio / 0.33;
        interpolate_color(EnkaiColors::CYAN_BRIGHT, EnkaiColors::BLUE_MEDIUM, local)
    } else if ratio < 0.66 {
        // Blue → Purple
        let local = (ratio - 0.33) / 0.33;
        interpolate_color(EnkaiColors::BLUE_MEDIUM, EnkaiColors::PURPLE_BRIGHT, local)
    } else {
        // Purple → Magenta
        let local = (ratio - 0.66) / 0.34;
        interpolate_color(EnkaiColors::PURPLE_BRIGHT, EnkaiColors::MAGENTA_BRIGHT, local)
    }
}

fn interpolate_color(from: Color, to: Color, ratio: f32) -> Color {
    let (r1, g1, b1) = match from {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (0.0, 0.0, 0.0),
    };

    let (r2, g2, b2) = match to {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (0.0, 0.0, 0.0),
    };

    let r = (r1 + (r2 - r1) * ratio) as u8;
    let g = (g1 + (g2 - g1) * ratio) as u8;
    let b = (b1 + (b2 - b1) * ratio) as u8;

    Color::Rgb(r, g, b)
}
