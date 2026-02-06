/// UI Renderer for kiosk display
///
/// Renders all views: task cards, action menus, confirmation dialogs,
/// history view, settings menu, dashboard, QR code.
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use embedded_graphics::pixelcolor::Rgb565;

use crate::display::FrameBuffer;
use crate::fonts::{self, BIG_NUM_HEIGHT, BIG_NUM_WIDTH, FONT_WIDTH};
use crate::models::{HistoryDisplayEntry, TaskDisplayData};
use crate::theme;
use crate::wifi::WiFiMode;

/// Renderer handles all UI drawing operations
pub struct Renderer;

impl Renderer {
    /// Clear screen with background color
    fn clear(fb: &mut FrameBuffer) {
        fb.clear_color(theme::BACKGROUND);
    }

    /// Draw text at position (simple bitmap font)
    fn draw_text(fb: &mut FrameBuffer, x: u32, y: u32, text: &str, color: Rgb565, scale: u32) {
        let char_width = (FONT_WIDTH + 1) * scale;
        let mut cursor_x = x;

        for ch in text.chars() {
            Self::draw_char(fb, cursor_x, y, ch, color, scale);
            cursor_x += char_width;
        }
    }

    /// Draw a single character
    fn draw_char(fb: &mut FrameBuffer, x: u32, y: u32, ch: char, color: Rgb565, scale: u32) {
        let bitmap = fonts::get_char_bitmap(ch);

        for (row, &bits) in bitmap.iter().enumerate() {
            for col in 0..FONT_WIDTH {
                if (bits >> (FONT_WIDTH - 1 - col)) & 1 == 1 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            fb.set_pixel(
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

    /// Draw large friendly number (uses smoother 12x18 font)
    fn draw_big_number(fb: &mut FrameBuffer, x: u32, y: u32, ch: char, color: Rgb565, scale: u32) {
        let bitmap = fonts::get_big_num_bitmap(ch);

        for (row, &bits) in bitmap.iter().enumerate() {
            for col in 0..BIG_NUM_WIDTH {
                if (bits >> (BIG_NUM_WIDTH - 1 - col)) & 1 == 1 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            fb.set_pixel(
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

    /// Draw large number string centered
    fn draw_big_number_centered(fb: &mut FrameBuffer, y: u32, text: &str, color: Rgb565, scale: u32) {
        let char_width = (BIG_NUM_WIDTH + 2) * scale;
        let total_width = text.len() as u32 * char_width;
        let start_x = (fb.width().saturating_sub(total_width)) / 2;

        let mut cursor_x = start_x;
        for ch in text.chars() {
            Self::draw_big_number(fb, cursor_x, y, ch, color, scale);
            cursor_x += char_width;
        }
    }

    /// Calculate big number width
    fn big_number_width(text: &str, scale: u32) -> u32 {
        text.len() as u32 * (BIG_NUM_WIDTH + 2) * scale
    }

    /// Calculate text width
    fn text_width(text: &str, scale: u32) -> u32 {
        text.len() as u32 * (FONT_WIDTH + 1) * scale
    }

    /// Draw centered text
    fn draw_text_centered(fb: &mut FrameBuffer, y: u32, text: &str, color: Rgb565, scale: u32) {
        let w = Self::text_width(text, scale);
        let x = (fb.width().saturating_sub(w)) / 2;
        Self::draw_text(fb, x, y, text, color, scale);
    }

    /// Draw a pill-shaped badge (rounded rectangle with text)
    fn draw_pill(fb: &mut FrameBuffer, y: u32, text: &str, text_color: Rgb565, bg_color: Rgb565, scale: u32) {
        let text_w = Self::text_width(text, scale);
        let padding_x: u32 = 5;
        let padding_y: u32 = 2;
        let pill_w = text_w + padding_x * 2;
        let pill_h = 7 * scale + padding_y * 2;
        let x = (fb.width().saturating_sub(pill_w)) / 2;

        // Draw the full rectangle first
        fb.fill_rect(x, y, pill_w, pill_h, bg_color);

        // Cut corners for rounded effect (remove 2x2 corner pixels)
        // Top-left
        fb.set_pixel(x, y, theme::BACKGROUND);
        fb.set_pixel(x + 1, y, theme::BACKGROUND);
        fb.set_pixel(x, y + 1, theme::BACKGROUND);
        // Top-right
        fb.set_pixel(x + pill_w - 1, y, theme::BACKGROUND);
        fb.set_pixel(x + pill_w - 2, y, theme::BACKGROUND);
        fb.set_pixel(x + pill_w - 1, y + 1, theme::BACKGROUND);
        // Bottom-left
        fb.set_pixel(x, y + pill_h - 1, theme::BACKGROUND);
        fb.set_pixel(x + 1, y + pill_h - 1, theme::BACKGROUND);
        fb.set_pixel(x, y + pill_h - 2, theme::BACKGROUND);
        // Bottom-right
        fb.set_pixel(x + pill_w - 1, y + pill_h - 1, theme::BACKGROUND);
        fb.set_pixel(x + pill_w - 2, y + pill_h - 1, theme::BACKGROUND);
        fb.set_pixel(x + pill_w - 1, y + pill_h - 2, theme::BACKGROUND);

        // Draw text centered in pill
        let text_x = x + padding_x;
        let text_y = y + padding_y;
        Self::draw_text(fb, text_x, text_y, text, text_color, scale);
    }

    /// Draw a button pill at specific position
    fn draw_button_pill(
        fb: &mut FrameBuffer,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        text: &str,
        bg_color: Rgb565,
        text_color: Rgb565,
    ) {
        // Draw filled rectangle
        fb.fill_rect(x, y, width, height, bg_color);

        // Cut corners for rounded effect
        // Top-left
        fb.set_pixel(x, y, theme::BACKGROUND);
        fb.set_pixel(x + 1, y, theme::BACKGROUND);
        fb.set_pixel(x, y + 1, theme::BACKGROUND);
        // Top-right
        fb.set_pixel(x + width - 1, y, theme::BACKGROUND);
        fb.set_pixel(x + width - 2, y, theme::BACKGROUND);
        fb.set_pixel(x + width - 1, y + 1, theme::BACKGROUND);
        // Bottom-left
        fb.set_pixel(x, y + height - 1, theme::BACKGROUND);
        fb.set_pixel(x + 1, y + height - 1, theme::BACKGROUND);
        fb.set_pixel(x, y + height - 2, theme::BACKGROUND);
        // Bottom-right
        fb.set_pixel(x + width - 1, y + height - 1, theme::BACKGROUND);
        fb.set_pixel(x + width - 2, y + height - 1, theme::BACKGROUND);
        fb.set_pixel(x + width - 1, y + height - 2, theme::BACKGROUND);

        // Center text in button
        let text_w = Self::text_width(text, 1);
        let text_x = x + (width.saturating_sub(text_w)) / 2;
        let text_y = y + (height.saturating_sub(7)) / 2;
        Self::draw_text(fb, text_x, text_y, text, text_color, 1);
    }

    /// Render a task card (main view)
    pub fn render_task_card(fb: &mut FrameBuffer, task: &TaskDisplayData, index: usize, total: usize) {
        Self::clear(fb);

        let h = fb.height();
        let w = fb.width();
        let max_chars_per_line = ((w - 8) / (FONT_WIDTH + 1)) as usize;

        // Urgency label at top with pill background
        let urgency_color = theme::urgency_color(&task.urgency);
        let urgency_label = theme::urgency_label(&task.urgency);
        Self::draw_pill(fb, 3, urgency_label, theme::TEXT_PRIMARY, urgency_color, 1);

        // Task name - wrap to multiple lines if needed
        let name_lines = wrap_text(&task.name, max_chars_per_line.min(25));
        let name_start_y = 16;
        for (i, line) in name_lines.iter().take(2).enumerate() {
            Self::draw_text_centered(fb, name_start_y + (i as u32 * 9), line, theme::TEXT_PRIMARY, 1);
        }

        // Large day count
        let days_text = format!("{}", task.days_until_due.unsigned_abs());

        // Big number in center
        let number_y = if name_lines.len() > 1 { 36 } else { 32 };

        // Use scale 2 for big friendly numbers
        // For 3+ digit numbers, use scale 1 to fit
        let scale = if days_text.len() >= 3 { 1 } else { 2 };
        Self::draw_big_number_centered(fb, number_y, &days_text, urgency_color, scale);

        // "DAYS LEFT" or "DAYS OVERDUE" label
        let days_label = if task.days_until_due < 0 {
            "DAYS OVERDUE"
        } else if task.days_until_due == 1 {
            "DAY LEFT"
        } else {
            "DAYS LEFT"
        };
        let number_height = BIG_NUM_HEIGHT * scale;
        let label_y = number_y + number_height + 2;
        Self::draw_text_centered(fb, label_y, days_label, theme::TEXT_MUTED, 1);

        // Due date
        Self::draw_text_centered(fb, label_y + 10, &task.next_due_date, theme::TEXT_MUTED, 1);

        // Navigation hint at bottom
        let nav_text = format!("<< {}/{} >>", index + 1, total);
        Self::draw_text_centered(fb, h - 9, &nav_text, theme::TEXT_MUTED, 1);
    }

    /// Render action menu
    pub fn render_action_menu(fb: &mut FrameBuffer, task_name: &str, selected: usize, options: &[&str]) {
        Self::clear(fb);

        let h = fb.height();
        let max_chars = 20;

        // Task name at top (wrap if needed)
        let name_lines = wrap_text(task_name, max_chars);
        for (i, line) in name_lines.iter().take(2).enumerate() {
            Self::draw_text_centered(fb, 4 + (i as u32 * 9), line, theme::TEXT_PRIMARY, 1);
        }

        // Separator line
        let sep_y = if name_lines.len() > 1 { 24 } else { 16 };
        fb.hline(10, sep_y, fb.width() - 20, theme::CARD_BORDER);

        // Menu options
        let start_y = sep_y + 8;
        let item_height: u32 = 14;

        for (i, option) in options.iter().enumerate() {
            let y = start_y + (i as u32 * item_height);
            let is_selected = i == selected;

            if is_selected {
                fb.fill_rect(4, y - 2, fb.width() - 8, item_height, theme::SELECTION_BG);
                Self::draw_text(fb, 8, y, ">", theme::ACCENT, 1);
            }

            let color = if is_selected { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };

            let text_color = match option.to_ascii_lowercase().as_str() {
                "delete" => theme::DESTRUCTIVE,
                "done" | "complete" => theme::SUCCESS,
                _ => color,
            };

            Self::draw_text(fb, 20, y, option, text_color, 1);
        }

        Self::draw_text_centered(fb, h - 10, "press to select", theme::TEXT_MUTED, 1);
    }

    /// Render confirmation dialog
    pub fn render_confirm_dialog(fb: &mut FrameBuffer, message: &str, confirm_selected: bool) {
        Self::clear(fb);

        let w = fb.width();
        let h = fb.height();

        // Warning icon
        Self::draw_text_centered(fb, 20, "!", theme::DESTRUCTIVE, 3);

        // Message (wrapped)
        let lines = wrap_text(message, 20);
        let start_y = 50;
        for (i, line) in lines.iter().enumerate() {
            Self::draw_text_centered(fb, start_y + (i as u32 * 10), line, theme::TEXT_PRIMARY, 1);
        }

        // Buttons
        let btn_y = h - 28;
        let btn_width: u32 = 52;
        let btn_height: u32 = 16;
        let gap: u32 = 16;

        let cancel_x = (w - btn_width * 2 - gap) / 2;
        let confirm_x = cancel_x + btn_width + gap;

        // Cancel button
        if !confirm_selected {
            Self::draw_button_pill(fb, cancel_x, btn_y, btn_width, btn_height, "Cancel", theme::SUCCESS, theme::TEXT_PRIMARY);
        } else {
            let text_x = cancel_x + (btn_width - Self::text_width("Cancel", 1)) / 2;
            Self::draw_text(fb, text_x, btn_y + 4, "Cancel", theme::TEXT_MUTED, 1);
        }

        // Delete button
        if confirm_selected {
            Self::draw_button_pill(fb, confirm_x, btn_y, btn_width, btn_height, "Delete", theme::DESTRUCTIVE, theme::TEXT_PRIMARY);
        } else {
            let text_x = confirm_x + (btn_width - Self::text_width("Delete", 1)) / 2;
            Self::draw_text(fb, text_x, btn_y + 4, "Delete", theme::TEXT_MUTED, 1);
        }
    }

    /// Render completing animation
    pub fn render_completing(fb: &mut FrameBuffer, task_name: &str, progress: f32) {
        Self::clear(fb);

        let w = fb.width();

        // Task name (wrapped)
        let name_lines = wrap_text(task_name, 20);
        for (i, line) in name_lines.iter().take(2).enumerate() {
            Self::draw_text_centered(fb, 20 + (i as u32 * 9), line, theme::TEXT_PRIMARY, 1);
        }

        if progress >= 1.0 {
            Self::draw_text_centered(fb, 55, "Done!", theme::SUCCESS, 2);
        } else {
            let bar_w = w - 40;
            let bar_h: u32 = 8;
            let bar_x: u32 = 20;
            let bar_y: u32 = 60;

            fb.fill_rect(bar_x, bar_y, bar_w, bar_h, theme::CARD_BORDER);

            let fill_w = ((bar_w as f32) * progress) as u32;
            fb.fill_rect(bar_x, bar_y, fill_w, bar_h, theme::SUCCESS);

            Self::draw_text_centered(fb, 80, "Completing...", theme::TEXT_MUTED, 1);
        }
    }

    /// Render history view
    pub fn render_history(fb: &mut FrameBuffer, task_name: &str, entries: &[HistoryDisplayEntry], selected: usize) {
        Self::clear(fb);

        let h = fb.height();

        Self::draw_text_centered(fb, 4, "History", theme::TEXT_PRIMARY, 1);

        // Task name (single line, truncated for history view)
        let name = if task_name.len() > 18 {
            let mut s = String::from(&task_name[..15]);
            s.push_str("...");
            s
        } else {
            String::from(task_name)
        };
        Self::draw_text_centered(fb, 14, &name, theme::TEXT_MUTED, 1);

        fb.hline(10, 24, fb.width() - 20, theme::CARD_BORDER);

        if entries.is_empty() {
            Self::draw_text_centered(fb, 50, "No history", theme::TEXT_MUTED, 1);
        } else {
            let max_visible = 6;
            let start_idx = if selected >= max_visible {
                selected - max_visible + 1
            } else {
                0
            };

            let item_height: u32 = 14;
            let start_y: u32 = 30;

            for (i, entry) in entries.iter().skip(start_idx).take(max_visible).enumerate() {
                let actual_idx = start_idx + i;
                let y = start_y + (i as u32 * item_height);
                let is_selected = actual_idx == selected;

                if is_selected {
                    fb.fill_rect(4, y - 2, fb.width() - 8, item_height, theme::SELECTION_BG);
                }

                let color = if is_selected { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };

                Self::draw_text(fb, 8, y, &entry.completed_at, color, 1);

                if let Some(days) = entry.days_since_last {
                    let days_text = format!("+{}", days);
                    let x = fb.width() - Self::text_width(&days_text, 1) - 8;
                    Self::draw_text(fb, x, y, &days_text, theme::TEXT_MUTED, 1);
                }
            }
        }

        Self::draw_text_centered(fb, h - 10, "long press: back", theme::TEXT_MUTED, 1);
    }

    /// Render settings menu
    pub fn render_settings(fb: &mut FrameBuffer, selected: usize, screen_timeout_enabled: bool) {
        Self::clear(fb);

        let h = fb.height();

        Self::draw_text_centered(fb, 4, "Settings", theme::TEXT_PRIMARY, 1);
        fb.hline(10, 16, fb.width() - 20, theme::CARD_BORDER);

        let start_y: u32 = 24;
        let item_height: u32 = 16;

        // Manage Tasks (index 0)
        let manage_y = start_y;
        let manage_selected = selected == 0;
        if manage_selected {
            fb.fill_rect(4, manage_y - 2, fb.width() - 8, item_height - 2, theme::SELECTION_BG);
            Self::draw_text(fb, 8, manage_y, ">", theme::ACCENT, 1);
        }
        let manage_color = if manage_selected { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
        Self::draw_text(fb, 20, manage_y, "Manage Tasks", manage_color, 1);
        let arrow_x = fb.width() - Self::text_width(">", 1) - 8;
        Self::draw_text(fb, arrow_x, manage_y, ">", theme::TEXT_MUTED, 1);

        // Screen Timeout (index 1)
        let timeout_y = start_y + item_height;
        let timeout_selected = selected == 1;
        if timeout_selected {
            fb.fill_rect(4, timeout_y - 2, fb.width() - 8, item_height - 2, theme::SELECTION_BG);
            Self::draw_text(fb, 8, timeout_y, ">", theme::ACCENT, 1);
        }
        let timeout_color = if timeout_selected { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
        Self::draw_text(fb, 20, timeout_y, "Screen Timeout", timeout_color, 1);
        let toggle_text = if screen_timeout_enabled { "[ON]" } else { "[OFF]" };
        let toggle_color = if screen_timeout_enabled { theme::SUCCESS } else { theme::TEXT_MUTED };
        let toggle_x = fb.width() - Self::text_width(toggle_text, 1) - 8;
        Self::draw_text(fb, toggle_x, timeout_y, toggle_text, toggle_color, 1);

        // Reset WiFi (index 2)
        let wifi_y = start_y + (2 * item_height);
        let wifi_selected = selected == 2;
        if wifi_selected {
            fb.fill_rect(4, wifi_y - 2, fb.width() - 8, item_height - 2, theme::SELECTION_BG);
            Self::draw_text(fb, 8, wifi_y, ">", theme::ACCENT, 1);
        }
        let wifi_color = if wifi_selected { theme::DESTRUCTIVE } else { theme::TEXT_MUTED };
        Self::draw_text(fb, 20, wifi_y, "Reset WiFi", wifi_color, 1);

        // Back (index 3)
        let back_y = start_y + (3 * item_height);
        let back_selected = selected == 3;
        if back_selected {
            fb.fill_rect(4, back_y - 2, fb.width() - 8, item_height - 2, theme::SELECTION_BG);
            Self::draw_text(fb, 8, back_y, ">", theme::ACCENT, 1);
        }
        let back_color = if back_selected { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
        Self::draw_text(fb, 20, back_y, "Back", back_color, 1);

        Self::draw_text_centered(fb, h - 10, "press to select", theme::TEXT_MUTED, 1);
    }

    /// Render empty state (mode-aware)
    pub fn render_empty(fb: &mut FrameBuffer, wifi_mode: &WiFiMode) {
        Self::clear(fb);

        Self::draw_text_centered(fb, 30, "No tasks", theme::TEXT_PRIMARY, 2);

        match wifi_mode {
            WiFiMode::Station { ip, .. } => {
                let url = crate::wifi::web_url_from_ip(*ip);
                Self::draw_text_centered(fb, 55, "Add tasks at:", theme::TEXT_MUTED, 1);
                Self::draw_text_centered(fb, 68, &url, theme::ACCENT, 1);
            }
            WiFiMode::AccessPoint { .. } => {
                Self::draw_text_centered(fb, 60, "Add tasks via web", theme::TEXT_MUTED, 1);
            }
        }

        Self::draw_text_centered(fb, fb.height() - 10, "press for QR code", theme::TEXT_MUTED, 1);
    }

    /// Render dashboard with metrics and navigation
    pub fn render_dashboard(
        fb: &mut FrameBuffer,
        overdue: u32,
        today: u32,
        week: u32,
        total: u32,
        selected: usize,
    ) {
        Self::clear(fb);

        let w = fb.width();
        let h = fb.height();

        // === URGENCY BAR (visual chart at top) ===
        let bar_y: u32 = 3;
        let bar_h: u32 = 12;
        let bar_margin: u32 = 6;
        let bar_w = w - (bar_margin * 2);

        // Draw bar background with border
        fb.fill_rect(bar_margin, bar_y, bar_w, bar_h, theme::CARD_BG);
        fb.hline(bar_margin, bar_y, bar_w, theme::CARD_BORDER);
        fb.hline(bar_margin, bar_y + bar_h - 1, bar_w, theme::CARD_BORDER);
        fb.vline(bar_margin, bar_y, bar_h, theme::CARD_BORDER);
        fb.vline(bar_margin + bar_w - 1, bar_y, bar_h, theme::CARD_BORDER);

        // Calculate proportions for stacked bar
        let inner_x = bar_margin + 1;
        let inner_y = bar_y + 1;
        let inner_w = bar_w - 2;
        let inner_h = bar_h - 2;

        if total > 0 {
            let overdue_w = (overdue as f32 / total as f32 * inner_w as f32) as u32;
            let today_w = (today as f32 / total as f32 * inner_w as f32) as u32;
            let week_only = week.saturating_sub(overdue).saturating_sub(today);
            let week_w = (week_only as f32 / total as f32 * inner_w as f32) as u32;

            let mut x = inner_x;

            if overdue_w > 0 {
                fb.fill_rect(x, inner_y, overdue_w, inner_h, theme::URGENCY_OVERDUE);
                x += overdue_w;
            }
            if today_w > 0 {
                fb.fill_rect(x, inner_y, today_w, inner_h, theme::URGENCY_TODAY);
                x += today_w;
            }
            if week_w > 0 {
                fb.fill_rect(x, inner_y, week_w, inner_h, theme::URGENCY_WEEK);
                x += week_w;
            }
            let remaining = (inner_x + inner_w).saturating_sub(x);
            if remaining > 0 {
                fb.fill_rect(x, inner_y, remaining, inner_h, theme::URGENCY_UPCOMING);
            }
        }

        // === 2x2 METRIC GRID ===
        let grid_y: u32 = 18;
        let cell_w = (w - 12) / 2;
        let cell_h: u32 = 38;
        let gap: u32 = 4;

        let col1_x: u32 = 4;
        let col2_x = col1_x + cell_w + gap;
        let row1_y = grid_y;
        let row2_y = grid_y + cell_h + gap;

        Self::draw_metric_cell(fb, col1_x, row1_y, cell_w, cell_h, "OVERDUE", overdue, theme::URGENCY_OVERDUE, selected == 0);
        Self::draw_metric_cell(fb, col2_x, row1_y, cell_w, cell_h, "TODAY", today, theme::URGENCY_TODAY, selected == 1);
        Self::draw_metric_cell(fb, col1_x, row2_y, cell_w, cell_h, "WEEK", week, theme::URGENCY_WEEK, selected == 2);
        Self::draw_metric_cell(fb, col2_x, row2_y, cell_w, cell_h, "TOTAL", total, theme::URGENCY_UPCOMING, selected == 3);

        // === NAVIGATION BAR ===
        let nav_y = h - 24;
        let btn_w: u32 = 65;
        let btn_h: u32 = 18;
        let nav_gap: u32 = 10;

        let all_x = (w - btn_w * 2 - nav_gap) / 2;
        let settings_x = all_x + btn_w + nav_gap;

        if selected == 4 {
            Self::draw_button_pill(fb, all_x, nav_y, btn_w, btn_h, "All Tasks", theme::ACCENT, theme::TEXT_PRIMARY);
            Self::draw_text(fb, settings_x + (btn_w - Self::text_width("Settings", 1)) / 2, nav_y + 5, "Settings", theme::TEXT_MUTED, 1);
        } else if selected == 5 {
            Self::draw_text(fb, all_x + (btn_w - Self::text_width("All Tasks", 1)) / 2, nav_y + 5, "All Tasks", theme::TEXT_MUTED, 1);
            Self::draw_button_pill(fb, settings_x, nav_y, btn_w, btn_h, "Settings", theme::ACCENT, theme::TEXT_PRIMARY);
        } else {
            Self::draw_text(fb, all_x + (btn_w - Self::text_width("All Tasks", 1)) / 2, nav_y + 5, "All Tasks", theme::TEXT_MUTED, 1);
            Self::draw_text(fb, settings_x + (btn_w - Self::text_width("Settings", 1)) / 2, nav_y + 5, "Settings", theme::TEXT_MUTED, 1);
        }
    }

    /// Draw a metric cell for the dashboard
    fn draw_metric_cell(
        fb: &mut FrameBuffer,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        label: &str,
        count: u32,
        color: Rgb565,
        selected: bool,
    ) {
        let bg_color = if selected { theme::SELECTION_BG } else { theme::CARD_BG };
        fb.fill_rect(x, y, w, h, bg_color);

        if selected {
            // Draw thick selection border
            fb.hline(x, y, w, color);
            fb.hline(x, y + 1, w, color);
            fb.hline(x, y + h - 1, w, color);
            fb.hline(x, y + h - 2, w, color);
            fb.vline(x, y, h, color);
            fb.vline(x + 1, y, h, color);
            fb.vline(x + w - 1, y, h, color);
            fb.vline(x + w - 2, y, h, color);
        }

        // Draw big number centered
        let num_str = format!("{}", count);
        let num_w = Self::big_number_width(&num_str, 1);
        let num_x = x + (w.saturating_sub(num_w)) / 2;
        let num_y = y + 6;

        for (i, ch) in num_str.chars().enumerate() {
            Self::draw_big_number(fb, num_x + i as u32 * (BIG_NUM_WIDTH + 2), num_y, ch, color, 1);
        }

        // Draw label below number
        let label_w = Self::text_width(label, 1);
        let label_x = x + (w.saturating_sub(label_w)) / 2;
        let label_y = y + h - 10;
        Self::draw_text(fb, label_x, label_y, label, theme::TEXT_MUTED, 1);
    }

    /// Render back card (for navigating back to dashboard)
    pub fn render_back_card(fb: &mut FrameBuffer, total_tasks: usize) {
        Self::clear(fb);

        let h = fb.height();

        // Arrow icon pointing left
        Self::draw_text_centered(fb, 30, "<", theme::ACCENT, 3);

        // "Back" text
        Self::draw_text_centered(fb, 60, "Back", theme::TEXT_PRIMARY, 2);

        // Subtitle
        Self::draw_text_centered(fb, 85, "to Dashboard", theme::TEXT_MUTED, 1);

        // Navigation hint at bottom
        let nav_text = format!("<< 0/{} >>", total_tasks);
        Self::draw_text_centered(fb, h - 12, &nav_text, theme::TEXT_MUTED, 1);
    }

    /// Render empty filtered list message
    pub fn render_empty_filtered(fb: &mut FrameBuffer, filter_name: &str) {
        Self::clear(fb);

        Self::draw_text_centered(fb, 35, "No tasks", theme::TEXT_PRIMARY, 2);

        let msg = match filter_name {
            "overdue" => "Nothing overdue!",
            "today" => "Nothing due today!",
            "week" => "Nothing this week!",
            _ => "No tasks found",
        };
        Self::draw_text_centered(fb, 65, msg, theme::SUCCESS, 1);

        Self::draw_text_centered(fb, fb.height() - 10, "long press: back", theme::TEXT_MUTED, 1);
    }

    /// Render QR code screen (mode-aware: WiFi QR in AP mode, URL QR in STA mode)
    pub fn render_qr_code(fb: &mut FrameBuffer, wifi_mode: &WiFiMode, url: &str) {
        use qrcode::QrCode;

        Self::clear(fb);

        let h = fb.height();
        let w = fb.width();

        // Choose QR data and header based on mode
        let (qr_data, header) = match wifi_mode {
            WiFiMode::AccessPoint { .. } => {
                (crate::wifi::wifi_qr_string(), "Scan to connect")
            }
            WiFiMode::Station { .. } => {
                (String::from(url), "Scan to open")
            }
        };

        Self::draw_text_centered(fb, 2, header, theme::TEXT_PRIMARY, 1);

        if let Ok(code) = QrCode::new(qr_data.as_bytes()) {
            let qr_size = code.width();
            let available = 86u32;
            let pixel_size = (available / qr_size as u32).max(1);
            let qr_pixels = qr_size as u32 * pixel_size;

            let start_x = (w - qr_pixels) / 2;
            let start_y: u32 = 14;

            // White background for QR
            fb.fill_rect(
                start_x.saturating_sub(4),
                start_y.saturating_sub(4),
                qr_pixels + 8,
                qr_pixels + 8,
                theme::TEXT_PRIMARY,
            );

            // QR modules
            for (y, row) in code.to_colors().chunks(qr_size).enumerate() {
                for (x, &color) in row.iter().enumerate() {
                    if color == qrcode::Color::Dark {
                        fb.fill_rect(
                            start_x + (x as u32 * pixel_size),
                            start_y + (y as u32 * pixel_size),
                            pixel_size,
                            pixel_size,
                            theme::BACKGROUND,
                        );
                    }
                }
            }

            // Show URL below QR code
            let url_y = start_y + qr_pixels + 10;
            Self::draw_text_centered(fb, url_y, url, theme::ACCENT, 1);
        }

        Self::draw_text_centered(fb, h - 10, "long press: back", theme::TEXT_MUTED, 1);
    }

    /// Render "connecting" splash screen
    pub fn render_connecting(fb: &mut FrameBuffer, message: &str) {
        Self::clear(fb);

        Self::draw_text_centered(fb, 50, message, theme::TEXT_PRIMARY, 1);
        Self::draw_text_centered(fb, 70, "Please wait...", theme::TEXT_MUTED, 1);
    }

    /// Render WiFi connection failure screen
    pub fn render_wifi_failed(fb: &mut FrameBuffer, ssid: &str) {
        Self::clear(fb);

        Self::draw_text_centered(fb, 20, "WiFi Failed", theme::DESTRUCTIVE, 2);

        let lines = wrap_text(ssid, 22);
        for (i, line) in lines.iter().take(2).enumerate() {
            Self::draw_text_centered(fb, 50 + (i as u32 * 10), line, theme::TEXT_MUTED, 1);
        }

        Self::draw_text_centered(fb, 80, "Restarting...", theme::TEXT_MUTED, 1);
    }

    /// Render Reset WiFi confirmation dialog
    pub fn render_reset_wifi_confirm(fb: &mut FrameBuffer, confirmed: bool) {
        Self::clear(fb);

        let w = fb.width();
        let h = fb.height();

        // Warning icon
        Self::draw_text_centered(fb, 15, "!", theme::DESTRUCTIVE, 3);

        Self::draw_text_centered(fb, 45, "Reset WiFi?", theme::TEXT_PRIMARY, 1);
        Self::draw_text_centered(fb, 58, "Device will restart", theme::TEXT_MUTED, 1);
        Self::draw_text_centered(fb, 68, "in setup mode", theme::TEXT_MUTED, 1);

        // Buttons
        let btn_y = h - 28;
        let btn_width: u32 = 52;
        let btn_height: u32 = 16;
        let gap: u32 = 16;

        let cancel_x = (w - btn_width * 2 - gap) / 2;
        let confirm_x = cancel_x + btn_width + gap;

        // Cancel button
        if !confirmed {
            Self::draw_button_pill(fb, cancel_x, btn_y, btn_width, btn_height, "Cancel", theme::SUCCESS, theme::TEXT_PRIMARY);
        } else {
            let text_x = cancel_x + (btn_width - Self::text_width("Cancel", 1)) / 2;
            Self::draw_text(fb, text_x, btn_y + 4, "Cancel", theme::TEXT_MUTED, 1);
        }

        // Reset button
        if confirmed {
            Self::draw_button_pill(fb, confirm_x, btn_y, btn_width, btn_height, "Reset", theme::DESTRUCTIVE, theme::TEXT_PRIMARY);
        } else {
            let text_x = confirm_x + (btn_width - Self::text_width("Reset", 1)) / 2;
            Self::draw_text(fb, text_x, btn_y + 4, "Reset", theme::TEXT_MUTED, 1);
        }
    }

    /// Render station mode "connected" splash
    pub fn render_connected(fb: &mut FrameBuffer, ssid: &str, url: &str) {
        Self::clear(fb);

        Self::draw_text_centered(fb, 20, "Connected!", theme::SUCCESS, 2);

        let lines = wrap_text(ssid, 22);
        for (i, line) in lines.iter().take(2).enumerate() {
            Self::draw_text_centered(fb, 50 + (i as u32 * 10), line, theme::TEXT_PRIMARY, 1);
        }

        Self::draw_text_centered(fb, 80, url, theme::ACCENT, 1);
        Self::draw_text_centered(fb, 100, "Starting...", theme::TEXT_MUTED, 1);
    }
}

/// Wrap text to multiple lines
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = String::from(word);
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = String::from(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Helper trait for lowercasing without std
trait ToAsciiLowercase {
    fn to_ascii_lowercase(&self) -> String;
}

impl ToAsciiLowercase for &str {
    fn to_ascii_lowercase(&self) -> String {
        self.chars().map(|c| c.to_ascii_lowercase()).collect()
    }
}
