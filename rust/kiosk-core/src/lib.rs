//! Kiosk Core - High-performance display and input handling for Days Tracker
//!
//! This Rust library provides:
//! - Direct framebuffer rendering for the 160x128 TFT display
//! - GPIO-based rotary encoder handling with microsecond latency
//! - Efficient UI rendering without browser overhead

mod display;
mod encoder;
mod renderer;
mod theme;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use display::Display;
pub use encoder::{Encoder, EncoderEvent};
pub use renderer::Renderer;
pub use theme::Theme;

/// Global flag for clean shutdown
static RUNNING: AtomicBool = AtomicBool::new(true);

/// Task data passed from Python for rendering
#[pyclass]
#[derive(Clone, Debug)]
pub struct TaskData {
    #[pyo3(get, set)]
    pub id: i64,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub days_until_due: i32,
    #[pyo3(get, set)]
    pub urgency: String,
    #[pyo3(get, set)]
    pub next_due_date: String,
}

#[pymethods]
impl TaskData {
    #[new]
    fn new(id: i64, name: String, days_until_due: i32, urgency: String, next_due_date: String) -> Self {
        Self { id, name, days_until_due, urgency, next_due_date }
    }
}

/// Completion history entry
#[pyclass]
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    #[pyo3(get, set)]
    pub completed_at: String,
    #[pyo3(get, set)]
    pub days_since_last: Option<i32>,
}

#[pymethods]
impl HistoryEntry {
    #[new]
    #[pyo3(signature = (completed_at, days_since_last=None))]
    fn new(completed_at: String, days_since_last: Option<i32>) -> Self {
        Self { completed_at, days_since_last }
    }
}

/// Main kiosk controller - owns display and encoder
#[pyclass]
pub struct KioskController {
    renderer: Renderer,
    encoder: Encoder,
    backlight_on: Arc<AtomicBool>,
}

#[pymethods]
impl KioskController {
    /// Create a new kiosk controller
    ///
    /// Args:
    ///     width: Display width in pixels (default 160)
    ///     height: Display height in pixels (default 128)
    ///     clk_pin: Encoder CLK GPIO pin (default 17)
    ///     dt_pin: Encoder DT GPIO pin (default 27)
    ///     sw_pin: Encoder switch GPIO pin (default 22)
    ///     bl_pin: Backlight GPIO pin (default 18)
    #[new]
    #[pyo3(signature = (width=160, height=128, clk_pin=17, dt_pin=27, sw_pin=22, bl_pin=18))]
    fn new(width: u32, height: u32, clk_pin: u8, dt_pin: u8, sw_pin: u8, bl_pin: u8) -> PyResult<Self> {
        let backlight_on = Arc::new(AtomicBool::new(true));

        let display = Display::new(width, height)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let renderer = Renderer::new(display);

        let encoder = Encoder::new(clk_pin, dt_pin, sw_pin, bl_pin, backlight_on.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self { renderer, encoder, backlight_on })
    }

    /// Poll for encoder events (non-blocking)
    /// Returns: "cw", "ccw", "press", "long_press", or None
    fn poll_encoder(&mut self) -> Option<String> {
        self.encoder.poll().map(|e| match e {
            EncoderEvent::Clockwise => "cw".to_string(),
            EncoderEvent::CounterClockwise => "ccw".to_string(),
            EncoderEvent::ShortPress => "press".to_string(),
            EncoderEvent::LongPress => "long_press".to_string(),
        })
    }

    /// Render a task card (main view)
    fn render_task_card(
        &mut self,
        task: &TaskData,
        index: usize,
        total: usize,
    ) -> PyResult<()> {
        self.renderer.render_task_card(task, index, total);
        Ok(())
    }

    /// Render the action menu for a task
    fn render_action_menu(
        &mut self,
        task_name: &str,
        selected_index: usize,
        options: Vec<String>,
    ) -> PyResult<()> {
        self.renderer.render_action_menu(task_name, selected_index, &options);
        Ok(())
    }

    /// Render delete confirmation dialog
    fn render_confirm_dialog(
        &mut self,
        message: &str,
        confirm_selected: bool,
    ) -> PyResult<()> {
        self.renderer.render_confirm_dialog(message, confirm_selected);
        Ok(())
    }

    /// Render completion animation
    fn render_completing(&mut self, task_name: &str, progress: f32) -> PyResult<()> {
        self.renderer.render_completing(task_name, progress);
        Ok(())
    }

    /// Render history view
    fn render_history(
        &mut self,
        task_name: &str,
        entries: Vec<HistoryEntry>,
        selected_index: usize,
    ) -> PyResult<()> {
        self.renderer.render_history(task_name, &entries, selected_index);
        Ok(())
    }

    /// Render settings menu
    fn render_settings(
        &mut self,
        selected_index: usize,
        screen_timeout_enabled: bool,
    ) -> PyResult<()> {
        self.renderer.render_settings(selected_index, screen_timeout_enabled);
        Ok(())
    }

    /// Render "no tasks" empty state
    fn render_empty(&mut self) -> PyResult<()> {
        self.renderer.render_empty();
        Ok(())
    }

    /// Render dashboard with metrics
    fn render_dashboard(
        &mut self,
        overdue: u32,
        today: u32,
        week: u32,
        total: u32,
        selected: usize,
    ) -> PyResult<()> {
        self.renderer.render_dashboard(overdue, today, week, total, selected);
        Ok(())
    }

    /// Render empty filtered list message
    fn render_empty_filtered(&mut self, filter_name: &str) -> PyResult<()> {
        self.renderer.render_empty_filtered(filter_name);
        Ok(())
    }

    /// Render back card for navigation
    fn render_back_card(&mut self, total_tasks: usize) -> PyResult<()> {
        self.renderer.render_back_card(total_tasks);
        Ok(())
    }

    /// Render QR code screen for web access
    fn render_qr_code(&mut self, url: &str) -> PyResult<()> {
        self.renderer.render_qr_code(url);
        Ok(())
    }

    /// Turn backlight on
    fn backlight_on(&mut self) -> PyResult<()> {
        self.encoder.set_backlight(true);
        self.backlight_on.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Turn backlight off
    fn backlight_off(&mut self) -> PyResult<()> {
        self.encoder.set_backlight(false);
        self.backlight_on.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Check if backlight is currently on
    fn is_backlight_on(&self) -> bool {
        self.backlight_on.load(Ordering::SeqCst)
    }

    /// Get seconds since last user activity
    fn seconds_since_activity(&self) -> f64 {
        self.encoder.seconds_since_activity()
    }

    /// Record user activity (resets idle timer)
    fn record_activity(&mut self) {
        self.encoder.record_activity();
    }
}

/// Check if the system should keep running
#[pyfunction]
fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

/// Signal shutdown
#[pyfunction]
fn shutdown() {
    RUNNING.store(false, Ordering::SeqCst);
}

/// Python module definition
#[pymodule]
fn kiosk_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<KioskController>()?;
    m.add_class::<TaskData>()?;
    m.add_class::<HistoryEntry>()?;
    m.add_function(wrap_pyfunction!(is_running, m)?)?;
    m.add_function(wrap_pyfunction!(shutdown, m)?)?;
    Ok(())
}
