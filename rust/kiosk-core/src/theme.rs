//! Theme colors and styling constants
//!
//! Warm, friendly theme designed for family kitchen display

use crate::display::Color;

/// Theme colors - warm and welcoming aesthetic
pub struct Theme;

impl Theme {
    // Background colors - warm cream tones
    pub const BACKGROUND: Color = Color::new(254, 249, 243);      // Warm cream #FEF9F3
    pub const CARD_BG: Color = Color::new(255, 255, 255);         // Pure white for cards
    pub const CARD_BORDER: Color = Color::new(235, 230, 225);     // Soft warm border

    // Text colors - warm charcoal, not harsh black
    pub const TEXT_PRIMARY: Color = Color::new(74, 74, 74);       // Warm charcoal #4A4A4A
    pub const TEXT_MUTED: Color = Color::new(139, 139, 139);      // Soft gray #8B8B8B

    // Friendly urgency colors - softer, more approachable
    pub const URGENCY_OVERDUE: Color = Color::new(255, 123, 123);   // Soft coral #FF7B7B
    pub const URGENCY_TODAY: Color = Color::new(255, 184, 107);     // Warm amber #FFB86B
    pub const URGENCY_TOMORROW: Color = Color::new(255, 204, 128);  // Soft gold #FFCC80
    pub const URGENCY_WEEK: Color = Color::new(125, 211, 168);      // Fresh mint #7DD3A8
    pub const URGENCY_UPCOMING: Color = Color::new(168, 180, 255);  // Soft lavender #A8B4FF

    // UI accent colors - friendly and colorful
    pub const ACCENT: Color = Color::new(100, 216, 203);          // Teal accent #64D8CB
    pub const DESTRUCTIVE: Color = Color::new(255, 123, 123);     // Soft coral for delete
    pub const SUCCESS: Color = Color::new(100, 216, 203);         // Teal for success

    // Selection highlight - subtle warmth
    pub const SELECTION_BG: Color = Color::new(245, 240, 235);    // Warm selection

    /// Get urgency color from string
    pub fn urgency_color(urgency: &str) -> Color {
        match urgency {
            "overdue" => Self::URGENCY_OVERDUE,
            "today" => Self::URGENCY_TODAY,
            "tomorrow" => Self::URGENCY_TOMORROW,
            "week" => Self::URGENCY_WEEK,
            _ => Self::URGENCY_UPCOMING,
        }
    }

    /// Get urgency label - friendlier language
    pub fn urgency_label(urgency: &str) -> &'static str {
        match urgency {
            "overdue" => "OVERDUE",
            "today" => "TODAY!",
            "tomorrow" => "TOMORROW",
            "week" => "THIS WEEK",
            _ => "COMING UP",
        }
    }
}
