//! UI Renderer for kiosk display
//!
//! Renders all views matching the original web UI layout:
//! - Task cards with countdown
//! - Action menus
//! - Confirmation dialogs
//! - History view
//! - Settings menu

use crate::display::{Color, Display};
use crate::theme::Theme;
use crate::{HistoryEntry, TaskData};

/// Simple bitmap font (5x7 pixels per character)
/// Characters are stored as 7 bytes each, MSB first
const FONT_WIDTH: u32 = 5;
const FONT_HEIGHT: u32 = 7;

/// Renderer handles all UI drawing operations
pub struct Renderer {
    display: Display,
}

impl Renderer {
    pub fn new(display: Display) -> Self {
        Self { display }
    }

    /// Clear screen with background color
    fn clear(&mut self) {
        self.display.clear(Theme::BACKGROUND);
    }

    /// Draw text at position (simple bitmap font)
    fn draw_text(&mut self, x: u32, y: u32, text: &str, color: Color, scale: u32) {
        let char_width = (FONT_WIDTH + 1) * scale;
        let mut cursor_x = x;

        for ch in text.chars() {
            self.draw_char(cursor_x, y, ch, color, scale);
            cursor_x += char_width;
        }
    }

    /// Draw a single character
    fn draw_char(&mut self, x: u32, y: u32, ch: char, color: Color, scale: u32) {
        let bitmap = get_char_bitmap(ch);

        for (row, &bits) in bitmap.iter().enumerate() {
            for col in 0..FONT_WIDTH {
                if (bits >> (FONT_WIDTH - 1 - col)) & 1 == 1 {
                    // Draw scaled pixel
                    for sy in 0..scale {
                        for sx in 0..scale {
                            self.display.set_pixel(
                                x + col * scale + sx,
                                y + row as u32 * scale + sy,
                                color,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Calculate text width
    fn text_width(&self, text: &str, scale: u32) -> u32 {
        text.len() as u32 * (FONT_WIDTH + 1) * scale
    }

    /// Draw centered text
    fn draw_text_centered(&mut self, y: u32, text: &str, color: Color, scale: u32) {
        let w = self.text_width(text, scale);
        let x = (self.display.width().saturating_sub(w)) / 2;
        self.draw_text(x, y, text, color, scale);
    }

    /// Render a task card (main view)
    pub fn render_task_card(&mut self, task: &TaskData, index: usize, total: usize) {
        self.clear();

        let h = self.display.height();

        // Urgency label at top
        let urgency_color = Theme::urgency_color(&task.urgency);
        let urgency_label = Theme::urgency_label(&task.urgency);
        self.draw_text_centered(4, urgency_label, urgency_color, 1);

        // Task name
        let name = truncate_text(&task.name, 16);
        self.draw_text_centered(16, &name, Theme::TEXT_PRIMARY, 1);

        // Large day count
        let days_text = if task.days_until_due < 0 {
            format!("{}", task.days_until_due.abs())
        } else {
            format!("{}", task.days_until_due)
        };

        // Big number in center
        let scale = if days_text.len() <= 2 { 4 } else { 3 };
        self.draw_text_centered(35, &days_text, urgency_color, scale);

        // "DAYS LEFT" or "DAYS OVERDUE" label
        let days_label = if task.days_until_due < 0 {
            "DAYS OVERDUE"
        } else if task.days_until_due == 1 {
            "DAY LEFT"
        } else {
            "DAYS LEFT"
        };
        self.draw_text_centered(70, days_label, Theme::TEXT_MUTED, 1);

        // Due date
        self.draw_text_centered(82, &task.next_due_date, Theme::TEXT_MUTED, 1);

        // Navigation hint at bottom
        let nav_text = format!("{}/{}", index + 1, total);
        self.draw_text_centered(h - 20, &nav_text, Theme::TEXT_MUTED, 1);

        // Scroll indicator
        self.draw_text_centered(h - 10, "scroll", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render action menu
    pub fn render_action_menu(&mut self, task_name: &str, selected: usize, options: &[String]) {
        self.clear();

        let h = self.display.height();

        // Task name at top
        let name = truncate_text(task_name, 16);
        self.draw_text_centered(4, &name, Theme::TEXT_PRIMARY, 1);

        // Separator line
        self.display.hline(10, 16, self.display.width() - 20, Theme::CARD_BORDER);

        // Menu options
        let start_y = 24;
        let item_height = 14;

        for (i, option) in options.iter().enumerate() {
            let y = start_y + (i as u32 * item_height);
            let is_selected = i == selected;

            if is_selected {
                // Highlight background
                self.display.fill_rect(4, y - 2, self.display.width() - 8, item_height, Theme::SELECTION_BG);
                // Selection indicator
                self.draw_text(8, y, ">", Theme::ACCENT, 1);
            }

            let color = if is_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };

            // Color code certain options
            let text_color = match option.to_lowercase().as_str() {
                "delete" => Theme::DESTRUCTIVE,
                "done" | "complete" => Theme::SUCCESS,
                _ => color,
            };

            self.draw_text(20, y, option, text_color, 1);
        }

        // Hint at bottom
        self.draw_text_centered(h - 10, "press to select", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render confirmation dialog
    pub fn render_confirm_dialog(&mut self, message: &str, confirm_selected: bool) {
        self.clear();

        let w = self.display.width();
        let h = self.display.height();

        // Warning icon area
        self.draw_text_centered(20, "!", Theme::DESTRUCTIVE, 3);

        // Message (may need to wrap)
        let lines = wrap_text(message, 20);
        let start_y = 50;
        for (i, line) in lines.iter().enumerate() {
            self.draw_text_centered(start_y + (i as u32 * 10), line, Theme::TEXT_PRIMARY, 1);
        }

        // Buttons
        let btn_y = h - 30;
        let btn_width = 50;
        let gap = 20;

        // Cancel button
        let cancel_x = (w - btn_width * 2 - gap) / 2;
        let cancel_color = if !confirm_selected { Theme::ACCENT } else { Theme::TEXT_MUTED };
        self.display.rect(cancel_x, btn_y, btn_width, 16, cancel_color);
        self.draw_text(cancel_x + 8, btn_y + 4, "Cancel", cancel_color, 1);

        // Confirm button
        let confirm_x = cancel_x + btn_width + gap;
        let confirm_color = if confirm_selected { Theme::DESTRUCTIVE } else { Theme::TEXT_MUTED };
        self.display.rect(confirm_x, btn_y, btn_width, 16, confirm_color);
        self.draw_text(confirm_x + 8, btn_y + 4, "Delete", confirm_color, 1);

        self.display.flush();
    }

    /// Render completing animation
    pub fn render_completing(&mut self, task_name: &str, progress: f32) {
        self.clear();

        let w = self.display.width();

        // Task name
        let name = truncate_text(task_name, 16);
        self.draw_text_centered(20, &name, Theme::TEXT_PRIMARY, 1);

        // Checkmark or progress
        if progress >= 1.0 {
            self.draw_text_centered(50, "Done!", Theme::SUCCESS, 2);
        } else {
            // Progress bar
            let bar_w = w - 40;
            let bar_h = 8;
            let bar_x = 20;
            let bar_y = 60;

            // Background
            self.display.fill_rect(bar_x, bar_y, bar_w, bar_h, Theme::CARD_BORDER);

            // Progress fill
            let fill_w = ((bar_w as f32) * progress) as u32;
            self.display.fill_rect(bar_x, bar_y, fill_w, bar_h, Theme::SUCCESS);

            self.draw_text_centered(80, "Completing...", Theme::TEXT_MUTED, 1);
        }

        self.display.flush();
    }

    /// Render history view
    pub fn render_history(&mut self, task_name: &str, entries: &[HistoryEntry], selected: usize) {
        self.clear();

        let h = self.display.height();

        // Header
        self.draw_text_centered(4, "History", Theme::TEXT_PRIMARY, 1);

        // Task name
        let name = truncate_text(task_name, 16);
        self.draw_text_centered(14, &name, Theme::TEXT_MUTED, 1);

        // Separator
        self.display.hline(10, 24, self.display.width() - 20, Theme::CARD_BORDER);

        if entries.is_empty() {
            self.draw_text_centered(50, "No history", Theme::TEXT_MUTED, 1);
        } else {
            // Show entries (max visible based on screen height)
            let max_visible = 6;
            let start_idx = if selected >= max_visible {
                selected - max_visible + 1
            } else {
                0
            };

            let item_height = 14;
            let start_y = 30;

            for (i, entry) in entries.iter().skip(start_idx).take(max_visible).enumerate() {
                let actual_idx = start_idx + i;
                let y = start_y + (i as u32 * item_height);
                let is_selected = actual_idx == selected;

                if is_selected {
                    self.display.fill_rect(4, y - 2, self.display.width() - 8, item_height, Theme::SELECTION_BG);
                }

                let color = if is_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };

                // Date
                self.draw_text(8, y, &entry.completed_at, color, 1);

                // Days since last (if available)
                if let Some(days) = entry.days_since_last {
                    let days_text = format!("+{}", days);
                    let x = self.display.width() - self.text_width(&days_text, 1) - 8;
                    self.draw_text(x, y, &days_text, Theme::TEXT_MUTED, 1);
                }
            }
        }

        // Hint
        self.draw_text_centered(h - 10, "long press: back", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render settings menu
    pub fn render_settings(&mut self, selected: usize, screen_timeout_enabled: bool) {
        self.clear();

        let h = self.display.height();

        // Header
        self.draw_text_centered(4, "Settings", Theme::TEXT_PRIMARY, 1);

        // Separator
        self.display.hline(10, 16, self.display.width() - 20, Theme::CARD_BORDER);

        let start_y = 30;
        let item_height = 18;

        // Item 0: Manage Tasks (no toggle, opens QR code)
        let manage_y = start_y;
        let manage_selected = selected == 0;
        if manage_selected {
            self.display.fill_rect(4, manage_y - 2, self.display.width() - 8, item_height - 2, Theme::SELECTION_BG);
            self.draw_text(8, manage_y, ">", Theme::ACCENT, 1);
        }
        let manage_color = if manage_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };
        self.draw_text(20, manage_y, "Manage Tasks", manage_color, 1);
        // Arrow indicator to show it opens something
        let arrow_x = self.display.width() - self.text_width(">", 1) - 8;
        self.draw_text(arrow_x, manage_y, ">", Theme::TEXT_MUTED, 1);

        // Item 1: Screen Timeout (toggle)
        let timeout_y = start_y + item_height;
        let timeout_selected = selected == 1;
        if timeout_selected {
            self.display.fill_rect(4, timeout_y - 2, self.display.width() - 8, item_height - 2, Theme::SELECTION_BG);
            self.draw_text(8, timeout_y, ">", Theme::ACCENT, 1);
        }
        let timeout_color = if timeout_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };
        self.draw_text(20, timeout_y, "Screen Timeout", timeout_color, 1);
        // Toggle indicator
        let toggle_text = if screen_timeout_enabled { "[ON]" } else { "[OFF]" };
        let toggle_color = if screen_timeout_enabled { Theme::SUCCESS } else { Theme::TEXT_MUTED };
        let toggle_x = self.display.width() - self.text_width(toggle_text, 1) - 8;
        self.draw_text(toggle_x, timeout_y, toggle_text, toggle_color, 1);

        // Item 2: Back
        let back_y = start_y + (2 * item_height);
        let back_selected = selected == 2;
        if back_selected {
            self.display.fill_rect(4, back_y - 2, self.display.width() - 8, item_height - 2, Theme::SELECTION_BG);
            self.draw_text(8, back_y, ">", Theme::ACCENT, 1);
        }
        let back_color = if back_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };
        self.draw_text(20, back_y, "Back", back_color, 1);

        // Hint
        self.draw_text_centered(h - 10, "press to select", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render empty state (no tasks)
    pub fn render_empty(&mut self) {
        self.clear();

        self.draw_text_centered(40, "No tasks", Theme::TEXT_PRIMARY, 2);
        self.draw_text_centered(70, "Add tasks via web", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render QR code screen for web access
    pub fn render_qr_code(&mut self, url: &str) {
        use qrcode::QrCode;

        self.clear();

        let h = self.display.height();
        let w = self.display.width();

        // Header
        self.draw_text_centered(2, "Scan to manage", Theme::TEXT_PRIMARY, 1);

        // Generate QR code
        if let Ok(code) = QrCode::new(url.as_bytes()) {
            let qr_size = code.width();

            // Calculate pixel size to fit on screen (leave room for text)
            // Available height: ~100px (128 - header - footer)
            // Available width: 160px
            let available = 96u32;
            let pixel_size = (available / qr_size as u32).max(1);
            let qr_pixels = qr_size as u32 * pixel_size;

            // Center the QR code
            let start_x = (w - qr_pixels) / 2;
            let start_y: u32 = 14;

            // Draw QR code with white background
            self.display.fill_rect(
                start_x.saturating_sub(4),
                start_y.saturating_sub(4),
                qr_pixels + 8,
                qr_pixels + 8,
                Theme::TEXT_PRIMARY,
            );

            // Draw QR modules
            for (y, row) in code.to_colors().chunks(qr_size).enumerate() {
                for (x, &color) in row.iter().enumerate() {
                    if color == qrcode::Color::Dark {
                        self.display.fill_rect(
                            start_x + (x as u32 * pixel_size),
                            start_y + (y as u32 * pixel_size),
                            pixel_size,
                            pixel_size,
                            Theme::BACKGROUND,
                        );
                    }
                }
            }
        }

        // Hint at bottom
        self.draw_text_centered(h - 10, "long press: back", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }
}

/// Truncate text to max characters with ellipsis
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Wrap text to multiple lines
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Get bitmap for a character (5x7 font)
fn get_char_bitmap(ch: char) -> [u8; 7] {
    match ch {
        ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '"' => [0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        '#' => [0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000],
        '$' => [0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100],
        '%' => [0b11001, 0b11010, 0b00100, 0b01000, 0b01011, 0b10011, 0b00000],
        '&' => [0b01100, 0b10010, 0b01100, 0b10101, 0b10010, 0b01101, 0b00000],
        '\'' => [0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '*' => [0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00000],
        '/' => [0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000, 0b00000],
        ';' => [0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        '<' => [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
        '=' => [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000],
        '>' => [0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000],
        '?' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100],
        '@' => [0b01110, 0b10001, 0b10111, 0b10101, 0b10110, 0b10000, 0b01110],
        'A' | 'a' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' | 'b' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' | 'c' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' | 'd' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' | 'e' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' | 'f' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' | 'g' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' | 'h' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' | 'i' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' | 'j' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' | 'k' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001],
        'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' | 'p' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' | 'q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' | 'r' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' | 's' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' | 'w' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' | 'x' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' | 'y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' | 'z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
        '\\' => [0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
        '^' => [0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111], // Unknown char box
    }
}
