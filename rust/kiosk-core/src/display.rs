//! Framebuffer display handling
//!
//! Provides direct access to the Linux framebuffer for rendering
//! on the 160x128 TFT display without X11 or browser overhead.

use framebuffer::{Framebuffer, KdMode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Framebuffer error: {0}")]
    Framebuffer(#[from] framebuffer::FramebufferError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// RGB565 color (16-bit, native format for most TFT displays)
#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to RGB565 format (16-bit)
    pub fn to_rgb565(&self) -> u16 {
        let r = (self.r as u16 >> 3) & 0x1F;
        let g = (self.g as u16 >> 2) & 0x3F;
        let b = (self.b as u16 >> 3) & 0x1F;
        (r << 11) | (g << 5) | b
    }

    /// Convert to RGB888 format (32-bit with padding)
    pub fn to_rgb888(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Display wrapper for framebuffer operations
pub struct Display {
    fb: Framebuffer,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
    line_length: u32,
    buffer: Vec<u8>,
}

impl Display {
    /// Create a new display instance
    pub fn new(width: u32, height: u32) -> Result<Self, DisplayError> {
        // Try to open framebuffer
        let fb = Framebuffer::new("/dev/fb0")
            .or_else(|_| Framebuffer::new("/dev/fb1"))?;

        // Get display info (use references to avoid moving)
        let actual_width = fb.var_screen_info.xres;
        let actual_height = fb.var_screen_info.yres;
        let bytes_per_pixel = fb.var_screen_info.bits_per_pixel / 8;
        let line_length = fb.fix_screen_info.line_length;

        println!("  Framebuffer: {}x{} @ {}bpp", actual_width, actual_height, bytes_per_pixel * 8);

        // Use requested dimensions or actual if smaller
        let width = width.min(actual_width);
        let height = height.min(actual_height);

        // Create buffer for double-buffering
        let buffer_size = (line_length * actual_height) as usize;
        let buffer = vec![0u8; buffer_size];

        // Switch console to graphics mode (disable cursor blinking)
        let _ = Framebuffer::set_kd_mode(KdMode::Graphics);

        Ok(Self {
            fb,
            width,
            height,
            bytes_per_pixel,
            line_length,
            buffer,
        })
    }

    /// Get display width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get display height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Clear the buffer with a color
    pub fn clear(&mut self, color: Color) {
        let bpp = self.bytes_per_pixel as usize;

        match bpp {
            2 => {
                // RGB565
                let pixel = color.to_rgb565();
                let bytes = pixel.to_le_bytes();
                for y in 0..self.height as usize {
                    let row_start = y * self.line_length as usize;
                    for x in 0..self.width as usize {
                        let offset = row_start + x * bpp;
                        self.buffer[offset] = bytes[0];
                        self.buffer[offset + 1] = bytes[1];
                    }
                }
            }
            3 | 4 => {
                // RGB888 or RGBA8888
                for y in 0..self.height as usize {
                    let row_start = y * self.line_length as usize;
                    for x in 0..self.width as usize {
                        let offset = row_start + x * bpp;
                        self.buffer[offset] = color.b;
                        self.buffer[offset + 1] = color.g;
                        self.buffer[offset + 2] = color.r;
                        if bpp == 4 {
                            self.buffer[offset + 3] = 255;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Set a pixel in the buffer
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let bpp = self.bytes_per_pixel as usize;
        let offset = (y as usize * self.line_length as usize) + (x as usize * bpp);

        match bpp {
            2 => {
                let pixel = color.to_rgb565();
                let bytes = pixel.to_le_bytes();
                self.buffer[offset] = bytes[0];
                self.buffer[offset + 1] = bytes[1];
            }
            3 | 4 => {
                self.buffer[offset] = color.b;
                self.buffer[offset + 1] = color.g;
                self.buffer[offset + 2] = color.r;
                if bpp == 4 {
                    self.buffer[offset + 3] = 255;
                }
            }
            _ => {}
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for py in y..y_end {
            for px in x..x_end {
                self.set_pixel(px, py, color);
            }
        }
    }

    /// Draw a horizontal line
    pub fn hline(&mut self, x: u32, y: u32, w: u32, color: Color) {
        self.fill_rect(x, y, w, 1, color);
    }

    /// Draw a vertical line
    pub fn vline(&mut self, x: u32, y: u32, h: u32, color: Color) {
        self.fill_rect(x, y, 1, h, color);
    }

    /// Draw a rectangle outline
    pub fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        self.hline(x, y, w, color);
        self.hline(x, y + h - 1, w, color);
        self.vline(x, y, h, color);
        self.vline(x + w - 1, y, h, color);
    }

    /// Flush buffer to display
    pub fn flush(&mut self) {
        let _ = self.fb.write_frame(&self.buffer);
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        // Restore text mode on exit
        let _ = Framebuffer::set_kd_mode(KdMode::Text);
    }
}
