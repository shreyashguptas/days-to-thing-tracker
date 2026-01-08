//! UI Renderer for kiosk display
//!
//! Renders all views:
//! - Task cards with countdown
//! - Action menus
//! - Confirmation dialogs
//! - History view
//! - Settings menu

use crate::display::{Color, Display};
use crate::theme::Theme;
use crate::{HistoryEntry, TaskData};

/// Bitmap font width (5 pixels per character)
const FONT_WIDTH: u32 = 5;

/// Large number font dimensions (smoother, friendlier numbers)
const BIG_NUM_WIDTH: u32 = 12;
const BIG_NUM_HEIGHT: u32 = 18;

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

    /// Draw large friendly number (uses smoother 12x18 font)
    fn draw_big_number(&mut self, x: u32, y: u32, ch: char, color: Color, scale: u32) {
        let bitmap = get_big_num_bitmap(ch);

        for (row, &bits) in bitmap.iter().enumerate() {
            for col in 0..BIG_NUM_WIDTH {
                if (bits >> (BIG_NUM_WIDTH - 1 - col)) & 1 == 1 {
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

    /// Draw large number string centered
    fn draw_big_number_centered(&mut self, y: u32, text: &str, color: Color, scale: u32) {
        let char_width = (BIG_NUM_WIDTH + 2) * scale;
        let total_width = text.len() as u32 * char_width;
        let start_x = (self.display.width().saturating_sub(total_width)) / 2;

        let mut cursor_x = start_x;
        for ch in text.chars() {
            self.draw_big_number(cursor_x, y, ch, color, scale);
            cursor_x += char_width;
        }
    }

    /// Calculate big number width
    fn big_number_width(&self, text: &str, scale: u32) -> u32 {
        text.len() as u32 * (BIG_NUM_WIDTH + 2) * scale
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

    /// Draw a pill-shaped badge (rounded rectangle with text)
    fn draw_pill(&mut self, y: u32, text: &str, text_color: Color, bg_color: Color, scale: u32) {
        let text_w = self.text_width(text, scale);
        let padding_x: u32 = 5;
        let padding_y: u32 = 2;
        let pill_w = text_w + padding_x * 2;
        let pill_h = 7 * scale + padding_y * 2;
        let x = (self.display.width().saturating_sub(pill_w)) / 2;

        // Simple rounded pill: main rectangle with corner pixels cut
        // Draw the full rectangle first
        self.display.fill_rect(x, y, pill_w, pill_h, bg_color);

        // Cut corners for rounded effect (remove 2x2 corner pixels)
        // Top-left
        self.display.set_pixel(x, y, Theme::BACKGROUND);
        self.display.set_pixel(x + 1, y, Theme::BACKGROUND);
        self.display.set_pixel(x, y + 1, Theme::BACKGROUND);
        // Top-right
        self.display.set_pixel(x + pill_w - 1, y, Theme::BACKGROUND);
        self.display.set_pixel(x + pill_w - 2, y, Theme::BACKGROUND);
        self.display.set_pixel(x + pill_w - 1, y + 1, Theme::BACKGROUND);
        // Bottom-left
        self.display.set_pixel(x, y + pill_h - 1, Theme::BACKGROUND);
        self.display.set_pixel(x + 1, y + pill_h - 1, Theme::BACKGROUND);
        self.display.set_pixel(x, y + pill_h - 2, Theme::BACKGROUND);
        // Bottom-right
        self.display.set_pixel(x + pill_w - 1, y + pill_h - 1, Theme::BACKGROUND);
        self.display.set_pixel(x + pill_w - 2, y + pill_h - 1, Theme::BACKGROUND);
        self.display.set_pixel(x + pill_w - 1, y + pill_h - 2, Theme::BACKGROUND);

        // Draw text centered in pill
        let text_x = x + padding_x;
        let text_y = y + padding_y;
        self.draw_text(text_x, text_y, text, text_color, scale);
    }

    /// Render a task card (main view)
    pub fn render_task_card(&mut self, task: &TaskData, index: usize, total: usize) {
        self.clear();

        let h = self.display.height();
        let w = self.display.width();
        // Use more of the screen width (was 22, now 26 chars)
        let max_chars_per_line = ((w - 8) / (FONT_WIDTH + 1)) as usize;

        // Urgency label at top with pill background
        let urgency_color = Theme::urgency_color(&task.urgency);
        let urgency_label = Theme::urgency_label(&task.urgency);
        self.draw_pill(3, urgency_label, Theme::TEXT_PRIMARY, urgency_color, 1);

        // Task name - wrap to multiple lines if needed (tighter spacing)
        let name_lines = wrap_text(&task.name, max_chars_per_line.min(25));
        let name_start_y = 16;
        for (i, line) in name_lines.iter().take(2).enumerate() {
            self.draw_text_centered(name_start_y + (i as u32 * 9), line, Theme::TEXT_PRIMARY, 1);
        }

        // Large day count - use friendly rounded numbers
        let days_text = format!("{}", task.days_until_due.abs());

        // Big number in center - tighter positioning
        let number_y = if name_lines.len() > 1 { 36 } else { 32 };

        // Use scale 2 for big friendly numbers (they're already 12x18 base)
        // For 3+ digit numbers, use scale 1 to fit
        let scale = if days_text.len() >= 3 { 1 } else { 2 };
        self.draw_big_number_centered(number_y, &days_text, urgency_color, scale);

        // "DAYS LEFT" or "DAYS OVERDUE" label
        let days_label = if task.days_until_due < 0 {
            "DAYS OVERDUE"
        } else if task.days_until_due == 1 {
            "DAY LEFT"
        } else {
            "DAYS LEFT"
        };
        // Big numbers are 18 pixels tall at scale 1, 36 at scale 2
        let number_height = BIG_NUM_HEIGHT * scale;
        let label_y = number_y + number_height + 2;
        self.draw_text_centered(label_y, days_label, Theme::TEXT_MUTED, 1);

        // Due date (tighter spacing - 10px instead of 12px)
        self.draw_text_centered(label_y + 10, &task.next_due_date, Theme::TEXT_MUTED, 1);

        // Navigation hint at bottom - combined single line with arrows
        let nav_text = format!("<< {}/{} >>", index + 1, total);
        self.draw_text_centered(h - 9, &nav_text, Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render action menu
    pub fn render_action_menu(&mut self, task_name: &str, selected: usize, options: &[String]) {
        self.clear();

        let h = self.display.height();
        let max_chars = 20;

        // Task name at top (wrap if needed)
        let name_lines = wrap_text(task_name, max_chars);
        for (i, line) in name_lines.iter().take(2).enumerate() {
            self.draw_text_centered(4 + (i as u32 * 9), line, Theme::TEXT_PRIMARY, 1);
        }

        // Separator line
        let sep_y = if name_lines.len() > 1 { 24 } else { 16 };
        self.display.hline(10, sep_y, self.display.width() - 20, Theme::CARD_BORDER);

        // Menu options
        let start_y = sep_y + 8;
        let item_height = 14;

        for (i, option) in options.iter().enumerate() {
            let y = start_y + (i as u32 * item_height);
            let is_selected = i == selected;

            if is_selected {
                self.display.fill_rect(4, y - 2, self.display.width() - 8, item_height, Theme::SELECTION_BG);
                self.draw_text(8, y, ">", Theme::ACCENT, 1);
            }

            let color = if is_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };

            let text_color = match option.to_lowercase().as_str() {
                "delete" => Theme::DESTRUCTIVE,
                "done" | "complete" => Theme::SUCCESS,
                _ => color,
            };

            self.draw_text(20, y, option, text_color, 1);
        }

        self.draw_text_centered(h - 10, "press to select", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Draw a button pill at specific position
    fn draw_button_pill(&mut self, x: u32, y: u32, width: u32, height: u32, text: &str, bg_color: Color, text_color: Color) {
        // Draw filled rectangle
        self.display.fill_rect(x, y, width, height, bg_color);

        // Cut corners for rounded effect
        // Top-left
        self.display.set_pixel(x, y, Theme::BACKGROUND);
        self.display.set_pixel(x + 1, y, Theme::BACKGROUND);
        self.display.set_pixel(x, y + 1, Theme::BACKGROUND);
        // Top-right
        self.display.set_pixel(x + width - 1, y, Theme::BACKGROUND);
        self.display.set_pixel(x + width - 2, y, Theme::BACKGROUND);
        self.display.set_pixel(x + width - 1, y + 1, Theme::BACKGROUND);
        // Bottom-left
        self.display.set_pixel(x, y + height - 1, Theme::BACKGROUND);
        self.display.set_pixel(x + 1, y + height - 1, Theme::BACKGROUND);
        self.display.set_pixel(x, y + height - 2, Theme::BACKGROUND);
        // Bottom-right
        self.display.set_pixel(x + width - 1, y + height - 1, Theme::BACKGROUND);
        self.display.set_pixel(x + width - 2, y + height - 1, Theme::BACKGROUND);
        self.display.set_pixel(x + width - 1, y + height - 2, Theme::BACKGROUND);

        // Center text in button
        let text_w = self.text_width(text, 1);
        let text_x = x + (width.saturating_sub(text_w)) / 2;
        let text_y = y + (height.saturating_sub(7)) / 2;
        self.draw_text(text_x, text_y, text, text_color, 1);
    }

    /// Render confirmation dialog
    pub fn render_confirm_dialog(&mut self, message: &str, confirm_selected: bool) {
        self.clear();

        let w = self.display.width();
        let h = self.display.height();

        // Warning icon
        self.draw_text_centered(20, "!", Theme::DESTRUCTIVE, 3);

        // Message (wrapped)
        let lines = wrap_text(message, 20);
        let start_y = 50;
        for (i, line) in lines.iter().enumerate() {
            self.draw_text_centered(start_y + (i as u32 * 10), line, Theme::TEXT_PRIMARY, 1);
        }

        // Buttons
        let btn_y = h - 28;
        let btn_width = 52;
        let btn_height = 16;
        let gap = 16;

        let cancel_x = (w - btn_width * 2 - gap) / 2;
        let confirm_x = cancel_x + btn_width + gap;

        // Cancel button - green pill when selected, muted text when not
        if !confirm_selected {
            self.draw_button_pill(cancel_x, btn_y, btn_width, btn_height, "Cancel", Theme::SUCCESS, Theme::TEXT_PRIMARY);
        } else {
            // Just draw muted text, no background
            let text_x = cancel_x + (btn_width - self.text_width("Cancel", 1)) / 2;
            self.draw_text(text_x, btn_y + 4, "Cancel", Theme::TEXT_MUTED, 1);
        }

        // Delete button - red pill when selected, muted text when not
        if confirm_selected {
            self.draw_button_pill(confirm_x, btn_y, btn_width, btn_height, "Delete", Theme::DESTRUCTIVE, Theme::TEXT_PRIMARY);
        } else {
            // Just draw muted text, no background
            let text_x = confirm_x + (btn_width - self.text_width("Delete", 1)) / 2;
            self.draw_text(text_x, btn_y + 4, "Delete", Theme::TEXT_MUTED, 1);
        }

        self.display.flush();
    }

    /// Render completing animation
    pub fn render_completing(&mut self, task_name: &str, progress: f32) {
        self.clear();

        let w = self.display.width();

        // Task name (wrapped)
        let name_lines = wrap_text(task_name, 20);
        for (i, line) in name_lines.iter().take(2).enumerate() {
            self.draw_text_centered(20 + (i as u32 * 9), line, Theme::TEXT_PRIMARY, 1);
        }

        if progress >= 1.0 {
            self.draw_text_centered(55, "Done!", Theme::SUCCESS, 2);
        } else {
            let bar_w = w - 40;
            let bar_h = 8;
            let bar_x = 20;
            let bar_y = 60;

            self.display.fill_rect(bar_x, bar_y, bar_w, bar_h, Theme::CARD_BORDER);

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

        self.draw_text_centered(4, "History", Theme::TEXT_PRIMARY, 1);

        // Task name (single line, truncated for history view)
        let name = if task_name.len() > 18 {
            format!("{}...", &task_name[..15])
        } else {
            task_name.to_string()
        };
        self.draw_text_centered(14, &name, Theme::TEXT_MUTED, 1);

        self.display.hline(10, 24, self.display.width() - 20, Theme::CARD_BORDER);

        if entries.is_empty() {
            self.draw_text_centered(50, "No history", Theme::TEXT_MUTED, 1);
        } else {
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

                self.draw_text(8, y, &entry.completed_at, color, 1);

                if let Some(days) = entry.days_since_last {
                    let days_text = format!("+{}", days);
                    let x = self.display.width() - self.text_width(&days_text, 1) - 8;
                    self.draw_text(x, y, &days_text, Theme::TEXT_MUTED, 1);
                }
            }
        }

        self.draw_text_centered(h - 10, "long press: back", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render settings menu
    pub fn render_settings(&mut self, selected: usize, screen_timeout_enabled: bool) {
        self.clear();

        let h = self.display.height();

        self.draw_text_centered(4, "Settings", Theme::TEXT_PRIMARY, 1);
        self.display.hline(10, 16, self.display.width() - 20, Theme::CARD_BORDER);

        let start_y = 30;
        let item_height = 18;

        // Manage Tasks
        let manage_y = start_y;
        let manage_selected = selected == 0;
        if manage_selected {
            self.display.fill_rect(4, manage_y - 2, self.display.width() - 8, item_height - 2, Theme::SELECTION_BG);
            self.draw_text(8, manage_y, ">", Theme::ACCENT, 1);
        }
        let manage_color = if manage_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };
        self.draw_text(20, manage_y, "Manage Tasks", manage_color, 1);
        let arrow_x = self.display.width() - self.text_width(">", 1) - 8;
        self.draw_text(arrow_x, manage_y, ">", Theme::TEXT_MUTED, 1);

        // Screen Timeout
        let timeout_y = start_y + item_height;
        let timeout_selected = selected == 1;
        if timeout_selected {
            self.display.fill_rect(4, timeout_y - 2, self.display.width() - 8, item_height - 2, Theme::SELECTION_BG);
            self.draw_text(8, timeout_y, ">", Theme::ACCENT, 1);
        }
        let timeout_color = if timeout_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };
        self.draw_text(20, timeout_y, "Screen Timeout", timeout_color, 1);
        let toggle_text = if screen_timeout_enabled { "[ON]" } else { "[OFF]" };
        let toggle_color = if screen_timeout_enabled { Theme::SUCCESS } else { Theme::TEXT_MUTED };
        let toggle_x = self.display.width() - self.text_width(toggle_text, 1) - 8;
        self.draw_text(toggle_x, timeout_y, toggle_text, toggle_color, 1);

        // Back
        let back_y = start_y + (2 * item_height);
        let back_selected = selected == 2;
        if back_selected {
            self.display.fill_rect(4, back_y - 2, self.display.width() - 8, item_height - 2, Theme::SELECTION_BG);
            self.draw_text(8, back_y, ">", Theme::ACCENT, 1);
        }
        let back_color = if back_selected { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED };
        self.draw_text(20, back_y, "Back", back_color, 1);

        self.draw_text_centered(h - 10, "press to select", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render empty state
    pub fn render_empty(&mut self) {
        self.clear();

        self.draw_text_centered(40, "No tasks", Theme::TEXT_PRIMARY, 2);
        self.draw_text_centered(70, "Add tasks via web", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render dashboard with metrics and navigation
    pub fn render_dashboard(
        &mut self,
        overdue: u32,
        today: u32,
        week: u32,
        total: u32,
        selected: usize,
    ) {
        self.clear();

        let w = self.display.width();
        let h = self.display.height();

        // === URGENCY BAR (visual chart at top) ===
        let bar_y = 4;
        let bar_h = 8;
        let bar_margin = 8;
        let bar_w = w - (bar_margin * 2);

        // Draw bar background
        self.display.fill_rect(bar_margin, bar_y, bar_w, bar_h, Theme::CARD_BORDER);

        // Calculate proportions for stacked bar
        if total > 0 {
            let overdue_w = (overdue as f32 / total as f32 * bar_w as f32) as u32;
            let today_w = (today as f32 / total as f32 * bar_w as f32) as u32;
            let week_only = week.saturating_sub(overdue).saturating_sub(today);
            let week_w = (week_only as f32 / total as f32 * bar_w as f32) as u32;

            let mut x = bar_margin;

            // Overdue segment (red)
            if overdue_w > 0 {
                self.display.fill_rect(x, bar_y, overdue_w, bar_h, Theme::URGENCY_OVERDUE);
                x += overdue_w;
            }
            // Today segment (orange)
            if today_w > 0 {
                self.display.fill_rect(x, bar_y, today_w, bar_h, Theme::URGENCY_TODAY);
                x += today_w;
            }
            // This week segment (green)
            if week_w > 0 {
                self.display.fill_rect(x, bar_y, week_w, bar_h, Theme::URGENCY_WEEK);
                x += week_w;
            }
            // Remaining (upcoming - blue)
            let remaining = bar_w.saturating_sub(x - bar_margin);
            if remaining > 0 {
                self.display.fill_rect(x, bar_y, remaining, bar_h, Theme::URGENCY_UPCOMING);
            }
        }

        // === 2x2 METRIC GRID ===
        let grid_y = 18;
        let cell_w = (w - 12) / 2;  // 2 columns with margins
        let cell_h = 38;
        let gap = 4;

        // Cell positions
        let col1_x = 4;
        let col2_x = col1_x + cell_w + gap;
        let row1_y = grid_y;
        let row2_y = grid_y + cell_h + gap;

        // Draw the 4 metric cells
        // 0 = OVERDUE, 1 = TODAY, 2 = WEEK, 3 = TOTAL
        self.draw_metric_cell(col1_x, row1_y, cell_w, cell_h, "OVERDUE", overdue, Theme::URGENCY_OVERDUE, selected == 0);
        self.draw_metric_cell(col2_x, row1_y, cell_w, cell_h, "TODAY", today, Theme::URGENCY_TODAY, selected == 1);
        self.draw_metric_cell(col1_x, row2_y, cell_w, cell_h, "WEEK", week, Theme::URGENCY_WEEK, selected == 2);
        self.draw_metric_cell(col2_x, row2_y, cell_w, cell_h, "TOTAL", total, Theme::URGENCY_UPCOMING, selected == 3);

        // === NAVIGATION BAR ===
        let nav_y = h - 24;
        let btn_w = 65;
        let btn_h = 18;
        let nav_gap = 10;

        let all_x = (w - btn_w * 2 - nav_gap) / 2;
        let settings_x = all_x + btn_w + nav_gap;

        // 4 = ALL_TASKS, 5 = SETTINGS
        if selected == 4 {
            self.draw_button_pill(all_x, nav_y, btn_w, btn_h, "All Tasks", Theme::ACCENT, Theme::TEXT_PRIMARY);
            self.draw_text(settings_x + (btn_w - self.text_width("Settings", 1)) / 2, nav_y + 5, "Settings", Theme::TEXT_MUTED, 1);
        } else if selected == 5 {
            self.draw_text(all_x + (btn_w - self.text_width("All Tasks", 1)) / 2, nav_y + 5, "All Tasks", Theme::TEXT_MUTED, 1);
            self.draw_button_pill(settings_x, nav_y, btn_w, btn_h, "Settings", Theme::ACCENT, Theme::TEXT_PRIMARY);
        } else {
            // Neither nav item selected, show both as muted
            self.draw_text(all_x + (btn_w - self.text_width("All Tasks", 1)) / 2, nav_y + 5, "All Tasks", Theme::TEXT_MUTED, 1);
            self.draw_text(settings_x + (btn_w - self.text_width("Settings", 1)) / 2, nav_y + 5, "Settings", Theme::TEXT_MUTED, 1);
        }

        // Navigation hint
        self.draw_text_centered(h - 6, "scroll to navigate", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Draw a metric cell for the dashboard
    fn draw_metric_cell(&mut self, x: u32, y: u32, w: u32, h: u32, label: &str, count: u32, color: Color, selected: bool) {
        // Draw cell background
        let bg_color = if selected { Theme::SELECTION_BG } else { Theme::CARD_BG };
        self.display.fill_rect(x, y, w, h, bg_color);

        // Draw selection border if selected
        if selected {
            // Top border
            self.display.hline(x, y, w, color);
            self.display.hline(x, y + 1, w, color);
            // Bottom border
            self.display.hline(x, y + h - 1, w, color);
            self.display.hline(x, y + h - 2, w, color);
            // Left border
            self.display.vline(x, y, h, color);
            self.display.vline(x + 1, y, h, color);
            // Right border
            self.display.vline(x + w - 1, y, h, color);
            self.display.vline(x + w - 2, y, h, color);
        }

        // Draw big number centered
        let num_str = count.to_string();
        let num_w = self.big_number_width(&num_str, 1);
        let num_x = x + (w.saturating_sub(num_w)) / 2;
        let num_y = y + 6;

        for (i, ch) in num_str.chars().enumerate() {
            self.draw_big_number(num_x + i as u32 * (BIG_NUM_WIDTH + 2), num_y, ch, color, 1);
        }

        // Draw label below number
        let label_w = self.text_width(label, 1);
        let label_x = x + (w.saturating_sub(label_w)) / 2;
        let label_y = y + h - 10;
        self.draw_text(label_x, label_y, label, Theme::TEXT_MUTED, 1);
    }

    /// Render empty filtered list message
    pub fn render_empty_filtered(&mut self, filter_name: &str) {
        self.clear();

        self.draw_text_centered(35, "No tasks", Theme::TEXT_PRIMARY, 2);

        let msg = match filter_name {
            "overdue" => "Nothing overdue!",
            "today" => "Nothing due today!",
            "week" => "Nothing this week!",
            _ => "No tasks found",
        };
        self.draw_text_centered(65, msg, Theme::SUCCESS, 1);

        self.draw_text_centered(self.display.height() - 10, "long press: back", Theme::TEXT_MUTED, 1);

        self.display.flush();
    }

    /// Render QR code screen
    pub fn render_qr_code(&mut self, url: &str) {
        use qrcode::QrCode;

        self.clear();

        let h = self.display.height();
        let w = self.display.width();

        self.draw_text_centered(2, "Scan to manage", Theme::TEXT_PRIMARY, 1);

        if let Ok(code) = QrCode::new(url.as_bytes()) {
            let qr_size = code.width();
            let available = 96u32;
            let pixel_size = (available / qr_size as u32).max(1);
            let qr_pixels = qr_size as u32 * pixel_size;

            let start_x = (w - qr_pixels) / 2;
            let start_y: u32 = 14;

            // White background for QR
            self.display.fill_rect(
                start_x.saturating_sub(4),
                start_y.saturating_sub(4),
                qr_pixels + 8,
                qr_pixels + 8,
                Theme::TEXT_PRIMARY,
            );

            // QR modules
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

        self.draw_text_centered(h - 10, "long press: back", Theme::TEXT_MUTED, 1);

        self.display.flush();
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
/// Numbers designed with rounded, friendly appearance
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
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110],
        '/' => [0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000],
        // Rounded, friendly numbers
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00110, 0b01110, 0b00110, 0b00110, 0b00110, 0b00110, 0b01111],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01100, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00011, 0b00101, 0b01001, 0b10001, 0b11111, 0b00001, 0b00001],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b10001, 0b00010, 0b00100, 0b00100, 0b00100, 0b00100],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00110, 0b00000],
        ';' => [0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00100, 0b01000],
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
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}

/// Get large bitmap for numbers (12x18 smooth, rounded font)
/// Designed to look friendly and modern, not robotic
fn get_big_num_bitmap(ch: char) -> [u16; 18] {
    match ch {
        '0' => [
            0b000111111000,
            0b001111111100,
            0b011110011110,
            0b011100001110,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b011100001110,
            0b011110011110,
            0b001111111100,
            0b000111111000,
            0b000000000000,
        ],
        '1' => [
            0b000011100000,
            0b000111100000,
            0b001111100000,
            0b011101100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b000001100000,
            0b001111111100,
            0b001111111100,
            0b000000000000,
        ],
        '2' => [
            0b000111111000,
            0b001111111100,
            0b011110011110,
            0b111000000111,
            0b000000000111,
            0b000000000111,
            0b000000001110,
            0b000000011100,
            0b000000111000,
            0b000001110000,
            0b000011100000,
            0b000111000000,
            0b001110000000,
            0b011100000000,
            0b111000000000,
            0b111111111111,
            0b111111111111,
            0b000000000000,
        ],
        '3' => [
            0b000111111000,
            0b001111111100,
            0b011110011110,
            0b111000000111,
            0b000000000111,
            0b000000000111,
            0b000000001110,
            0b000011111100,
            0b000011111100,
            0b000000001110,
            0b000000000111,
            0b000000000111,
            0b000000000111,
            0b111000000111,
            0b011110011110,
            0b001111111100,
            0b000111111000,
            0b000000000000,
        ],
        '4' => [
            0b000000011100,
            0b000000111100,
            0b000001111100,
            0b000011101100,
            0b000111001100,
            0b001110001100,
            0b011100001100,
            0b111000001100,
            0b111111111111,
            0b111111111111,
            0b000000001100,
            0b000000001100,
            0b000000001100,
            0b000000001100,
            0b000000001100,
            0b000000001100,
            0b000000001100,
            0b000000000000,
        ],
        '5' => [
            0b111111111111,
            0b111111111111,
            0b111000000000,
            0b111000000000,
            0b111000000000,
            0b111111111000,
            0b111111111100,
            0b000000011110,
            0b000000000111,
            0b000000000111,
            0b000000000111,
            0b000000000111,
            0b000000000111,
            0b111000000111,
            0b011110011110,
            0b001111111100,
            0b000111111000,
            0b000000000000,
        ],
        '6' => [
            0b000011111000,
            0b000111111100,
            0b001111001110,
            0b011110000000,
            0b011100000000,
            0b111000000000,
            0b111011111000,
            0b111111111100,
            0b111110011110,
            0b111100000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b011100001110,
            0b011110011110,
            0b001111111100,
            0b000111111000,
            0b000000000000,
        ],
        '7' => [
            0b111111111111,
            0b111111111111,
            0b000000000111,
            0b000000001110,
            0b000000011100,
            0b000000111000,
            0b000001110000,
            0b000001110000,
            0b000011100000,
            0b000011100000,
            0b000111000000,
            0b000111000000,
            0b000111000000,
            0b000111000000,
            0b000111000000,
            0b000111000000,
            0b000111000000,
            0b000000000000,
        ],
        '8' => [
            0b000111111000,
            0b001111111100,
            0b011110011110,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b011100001110,
            0b001111111100,
            0b001111111100,
            0b011100001110,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b011110011110,
            0b001111111100,
            0b000111111000,
            0b000000000000,
        ],
        '9' => [
            0b000111111000,
            0b001111111100,
            0b011110011110,
            0b011100001110,
            0b111000000111,
            0b111000000111,
            0b111000000111,
            0b111000001111,
            0b011110011111,
            0b001111111111,
            0b000111110111,
            0b000000000111,
            0b000000000111,
            0b000000001110,
            0b011100011100,
            0b001111111000,
            0b000111110000,
            0b000000000000,
        ],
        // Minus sign for negative numbers
        '-' => [
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b111111111111,
            0b111111111111,
            0b111111111111,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
            0b000000000000,
        ],
        // Fallback - empty
        _ => [0; 18],
    }
}
