/// Theme colors and styling constants
///
/// Dark theme with friendly accent colors for kitchen display
use embedded_graphics::pixelcolor::Rgb565;

/// Helper to convert 8-bit RGB to Rgb565
const fn rgb(r: u8, g: u8, b: u8) -> Rgb565 {
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

// Background colors - dark theme
pub const BACKGROUND: Rgb565 = rgb(15, 15, 15);           // Near black #0F0F0F
pub const CARD_BG: Rgb565 = rgb(25, 25, 25);              // Dark gray for cards
pub const CARD_BORDER: Rgb565 = rgb(45, 45, 45);          // Subtle border

// Text colors - bright and readable
pub const TEXT_PRIMARY: Rgb565 = rgb(255, 255, 255);       // Pure white
pub const TEXT_MUTED: Rgb565 = rgb(140, 140, 140);         // Soft gray

// Friendly urgency colors - vibrant but not harsh
pub const URGENCY_OVERDUE: Rgb565 = rgb(255, 107, 107);   // Soft red #FF6B6B
pub const URGENCY_TODAY: Rgb565 = rgb(255, 159, 67);       // Warm orange #FF9F43
pub const URGENCY_TOMORROW: Rgb565 = rgb(255, 206, 84);   // Sunny yellow #FFCE54
pub const URGENCY_WEEK: Rgb565 = rgb(46, 213, 115);       // Fresh green #2ED573
pub const URGENCY_UPCOMING: Rgb565 = rgb(116, 185, 255);  // Sky blue #74B9FF

// UI accent colors
pub const ACCENT: Rgb565 = rgb(99, 205, 218);             // Teal accent #63CDDA
pub const DESTRUCTIVE: Rgb565 = rgb(255, 107, 107);       // Soft red
pub const SUCCESS: Rgb565 = rgb(46, 213, 115);            // Fresh green

// Selection highlight
pub const SELECTION_BG: Rgb565 = rgb(40, 40, 40);         // Subtle highlight

/// Get urgency color from string
pub fn urgency_color(urgency: &str) -> Rgb565 {
    match urgency {
        "overdue" => URGENCY_OVERDUE,
        "today" => URGENCY_TODAY,
        "tomorrow" => URGENCY_TOMORROW,
        "week" => URGENCY_WEEK,
        _ => URGENCY_UPCOMING,
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
