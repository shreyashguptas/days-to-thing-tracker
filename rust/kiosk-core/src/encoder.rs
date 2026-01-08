//! Rotary encoder handling with GPIO
//!
//! Provides low-latency input handling for the KY-040 rotary encoder:
//! - Clockwise/counter-clockwise rotation detection
//! - Short press / long press differentiation
//! - Backlight control via GPIO

use rppal::gpio::{Gpio, InputPin, OutputPin, Level};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

/// Long press threshold in seconds
const LONG_PRESS_TIME: f64 = 0.5;

/// Debounce time for button in seconds
const BUTTON_DEBOUNCE: f64 = 0.2;

#[derive(Error, Debug)]
pub enum EncoderError {
    #[error("GPIO error: {0}")]
    Gpio(#[from] rppal::gpio::Error),
}

/// Events produced by the encoder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderEvent {
    Clockwise,
    CounterClockwise,
    ShortPress,
    LongPress,
}

/// Rotary encoder with button and backlight control
pub struct Encoder {
    clk: InputPin,
    dt: InputPin,
    sw: InputPin,
    backlight: Option<OutputPin>,
    last_clk: Level,
    button_press_time: Option<Instant>,
    last_button_time: Instant,
    last_activity: Instant,
    backlight_on: Arc<AtomicBool>,
}

impl Encoder {
    /// Create a new encoder instance
    pub fn new(
        clk_pin: u8,
        dt_pin: u8,
        sw_pin: u8,
        bl_pin: u8,
        backlight_on: Arc<AtomicBool>,
    ) -> Result<Self, EncoderError> {
        let gpio = Gpio::new()?;

        let clk = gpio.get(clk_pin)?.into_input_pullup();
        let dt = gpio.get(dt_pin)?.into_input_pullup();
        let sw = gpio.get(sw_pin)?.into_input_pullup();

        // Try to initialize backlight GPIO (may fail if already in use)
        let backlight = gpio.get(bl_pin)
            .map(|p| p.into_output_high())
            .ok();

        if backlight.is_some() {
            println!("  Backlight GPIO {} initialized", bl_pin);
        } else {
            println!("  Backlight GPIO {} not available", bl_pin);
        }

        let now = Instant::now();

        Ok(Self {
            clk,
            dt,
            sw,
            backlight,
            last_clk: Level::High,
            button_press_time: None,
            last_button_time: now,
            last_activity: now,
            backlight_on,
        })
    }

    /// Poll for encoder events (non-blocking)
    pub fn poll(&mut self) -> Option<EncoderEvent> {
        // Check rotation
        let clk_state = self.clk.read();

        // Detect falling edge on CLK
        if clk_state == Level::Low && self.last_clk == Level::High {
            self.last_clk = clk_state;
            self.record_activity();

            // DT high = clockwise, DT low = counter-clockwise
            return Some(if self.dt.read() == Level::High {
                EncoderEvent::Clockwise
            } else {
                EncoderEvent::CounterClockwise
            });
        }
        self.last_clk = clk_state;

        // Check button state
        let button_pressed = self.sw.read() == Level::Low;
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

    /// Set backlight state
    pub fn set_backlight(&mut self, on: bool) {
        if let Some(ref mut bl) = self.backlight {
            if on {
                bl.set_high();
            } else {
                bl.set_low();
            }
        }
    }

    /// Record user activity
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();

        // Wake up screen if it was off
        if !self.backlight_on.load(Ordering::SeqCst) {
            self.set_backlight(true);
            self.backlight_on.store(true, Ordering::SeqCst);
        }
    }

    /// Get seconds since last activity
    pub fn seconds_since_activity(&self) -> f64 {
        self.last_activity.elapsed().as_secs_f64()
    }
}
