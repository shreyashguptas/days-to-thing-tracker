/// Rotary encoder handling with ESP-IDF GPIO
///
/// Provides low-latency input handling for the KY-040 rotary encoder:
/// - Clockwise/counter-clockwise rotation detection
/// - Short press / long press differentiation
/// - Backlight control via GPIO
use esp_idf_hal::gpio::{Input, InputPin, Output, OutputPin, Pin, PinDriver, Pull};
use esp_idf_hal::peripheral::Peripheral;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Long press threshold in seconds
const LONG_PRESS_TIME: f64 = 0.5;

/// Debounce time for button in seconds
const BUTTON_DEBOUNCE: f64 = 0.2;

/// Events produced by the encoder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderEvent {
    Clockwise,
    CounterClockwise,
    ShortPress,
    LongPress,
}

/// Rotary encoder with button and backlight control
pub struct Encoder<'d, CLK: Pin, DT: Pin, SW: Pin, BL: Pin> {
    clk: PinDriver<'d, CLK, Input>,
    dt: PinDriver<'d, DT, Input>,
    sw: PinDriver<'d, SW, Input>,
    backlight: PinDriver<'d, BL, Output>,
    last_clk: bool,
    button_press_time: Option<Instant>,
    last_button_time: Instant,
    last_activity: Instant,
    backlight_on: Arc<AtomicBool>,
}

impl<'d, CLK: InputPin + OutputPin, DT: InputPin + OutputPin, SW: InputPin + OutputPin, BL: OutputPin> Encoder<'d, CLK, DT, SW, BL> {
    /// Create a new encoder instance
    pub fn new(
        clk_pin: impl Peripheral<P = CLK> + 'd,
        dt_pin: impl Peripheral<P = DT> + 'd,
        sw_pin: impl Peripheral<P = SW> + 'd,
        bl_pin: impl Peripheral<P = BL> + 'd,
        backlight_on: Arc<AtomicBool>,
    ) -> Result<Self, esp_idf_hal::sys::EspError> {
        let mut clk = PinDriver::input(clk_pin)?;
        clk.set_pull(Pull::Up)?;

        let mut dt = PinDriver::input(dt_pin)?;
        dt.set_pull(Pull::Up)?;

        let mut sw = PinDriver::input(sw_pin)?;
        sw.set_pull(Pull::Up)?;

        let mut backlight = PinDriver::output(bl_pin)?;
        backlight.set_high()?; // Active-high: HIGH = backlight ON

        let now = Instant::now();

        Ok(Self {
            clk,
            dt,
            sw,
            backlight,
            last_clk: true, // Pull-up, so high is default
            button_press_time: None,
            last_button_time: now,
            last_activity: now,
            backlight_on,
        })
    }

    /// Poll for encoder events (non-blocking)
    pub fn poll(&mut self) -> Option<EncoderEvent> {
        // Check rotation
        let clk_state = self.clk.is_high();

        // Detect falling edge on CLK
        if !clk_state && self.last_clk {
            self.last_clk = clk_state;
            self.record_activity();

            // DT high = clockwise, DT low = counter-clockwise
            return Some(if self.dt.is_high() {
                EncoderEvent::Clockwise
            } else {
                EncoderEvent::CounterClockwise
            });
        }
        self.last_clk = clk_state;

        // Check button state (active low with pull-up)
        let button_pressed = self.sw.is_low();
        let now = Instant::now();

        match (button_pressed, self.button_press_time) {
            // Button just pressed
            (true, None) => {
                self.button_press_time = Some(now);
                self.record_activity();
            }
            // Button just released
            (false, Some(press_time)) => {
                self.button_press_time = None;

                // Debounce check
                if now.duration_since(self.last_button_time).as_secs_f64() < BUTTON_DEBOUNCE {
                    return None;
                }
                self.last_button_time = now;

                let duration = now.duration_since(press_time).as_secs_f64();
                return Some(if duration >= LONG_PRESS_TIME {
                    EncoderEvent::LongPress
                } else {
                    EncoderEvent::ShortPress
                });
            }
            _ => {}
        }

        None
    }

    /// Set backlight state (active-high: HIGH = on, LOW = off)
    pub fn set_backlight(&mut self, on: bool) {
        if on {
            let _ = self.backlight.set_high();
        } else {
            let _ = self.backlight.set_low();
        }
        self.backlight_on.store(on, Ordering::SeqCst);
    }

    /// Record user activity
    fn record_activity(&mut self) {
        self.last_activity = Instant::now();

        // Wake up screen if it was off
        if !self.backlight_on.load(Ordering::SeqCst) {
            self.set_backlight(true);
        }
    }

    /// Get seconds since last activity
    pub fn seconds_since_activity(&self) -> f64 {
        self.last_activity.elapsed().as_secs_f64()
    }

    /// Check if backlight is currently on
    pub fn is_backlight_on(&self) -> bool {
        self.backlight_on.load(Ordering::SeqCst)
    }
}
