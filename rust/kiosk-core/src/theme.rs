//! Theme colors and styling constants
//!
//! Dark theme with friendly accent colors for kitchen display

use crate::display::Color;

/// Theme colors - dark background with colorful accents
pub struct Theme;

impl Theme {
    // Background colors - dark theme
    pub const BACKGROUND: Color = Color::new(15, 15, 15);         // Near black #0F0F0F
    pub const CARD_BG: Color = Color::new(25, 25, 25);            // Dark gray for cards
    pub const CARD_BORDER: Color = Color::new(45, 45, 45);        // Subtle border

    // Text colors - bright and readable
    pub const TEXT_PRIMARY: Color = Color::new(255, 255, 255);    // Pure white
    pub const TEXT_MUTED: Color = Color::new(140, 140, 140);      // Soft gray

    // Friendly urgency colors - vibrant but not harsh
    pub const URGENCY_OVERDUE: Color = Color::new(255, 107, 107);   // Soft red #FF6B6B
    pub const URGENCY_TODAY: Color = Color::new(255, 159, 67);      // Warm orange #FF9F43
    pub const URGENCY_TOMORROW: Color = Color::new(255, 206, 84);   // Sunny yellow #FFCE54
    pub const URGENCY_WEEK: Color = Color::new(46, 213, 115);       // Fresh green #2ED573
    pub const URGENCY_UPCOMING: Color = Color::new(116, 185, 255);  // Sky blue #74B9FF

    // UI accent colors
    pub const ACCENT: Color = Color::new(99, 205, 218);           // Teal accent #63CDDA
    pub const DESTRUCTIVE: Color = Color::new(255, 107, 107);     // Soft red
    pub const SUCCESS: Color = Color::new(46, 213, 115);          // Fresh green

    // Selection highlight
    pub const SELECTION_BG: Color = Color::new(40, 40, 40);       // Subtle highlight

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

    /// Get urgency label
    pub fn urgency_label(urgency: &str) -> &'static str {
        match urgency {
            "overdue" => "OVERDUE",
            "today" => "TODAY",
            "tomorrow" => "TOMORROW",
            "week" => "THIS WEEK",
            _ => "UPCOMING",
        }
    }
}
