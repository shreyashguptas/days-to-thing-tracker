//! Theme colors and styling constants
//!
//! Matches the dark theme from the original web UI

use crate::display::Color;

/// Theme colors matching the original CSS variables
pub struct Theme;

impl Theme {
    // Background colors
    pub const BACKGROUND: Color = Color::new(10, 10, 10);       // --background: #0a0a0a
    pub const CARD_BG: Color = Color::new(23, 23, 23);          // --card: #171717
    pub const CARD_BORDER: Color = Color::new(38, 38, 38);      // --border: #262626

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::new(250, 250, 250);  // --foreground: #fafafa
    pub const TEXT_MUTED: Color = Color::new(163, 163, 163);    // --muted-foreground: #a3a3a3

    // Urgency colors
    pub const URGENCY_OVERDUE: Color = Color::new(239, 68, 68);   // --urgency-overdue: #ef4444
    pub const URGENCY_TODAY: Color = Color::new(249, 115, 22);    // --urgency-today: #f97316
    pub const URGENCY_TOMORROW: Color = Color::new(234, 179, 8);  // --urgency-tomorrow: #eab308
    pub const URGENCY_WEEK: Color = Color::new(34, 197, 94);      // --urgency-week: #22c55e
    pub const URGENCY_UPCOMING: Color = Color::new(163, 163, 163); // Same as muted

    // UI colors
    pub const ACCENT: Color = Color::new(59, 130, 246);         // Blue accent
    pub const DESTRUCTIVE: Color = Color::new(239, 68, 68);     // Red for delete
    pub const SUCCESS: Color = Color::new(34, 197, 94);         // Green for success

    // Selection highlight
    pub const SELECTION_BG: Color = Color::new(38, 38, 38);     // Darker selection

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
