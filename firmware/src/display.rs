/// Display driver for ST7735 SPI display
///
/// Provides FrameBuffer with DrawTarget implementation and SPI display init.
/// Replaces the Linux framebuffer approach with direct SPI display control.
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::Rgb565,
    Pixel,
};

use crate::config::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// Framebuffer for 160x128 display
/// Implements DrawTarget so embedded-graphics can draw to it
pub struct FrameBuffer {
    buf: Box<[Rgb565; (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize]>,
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            buf: Box::new([Rgb565::new(0, 0, 0); (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize]),
        }
    }

    /// Get raw pixel data as u16 slice for SPI transfer
    pub fn as_raw(&self) -> &[u16] {
        // SAFETY: Rgb565 is repr(transparent) over u16, so this is a valid reinterpret
        unsafe {
            core::slice::from_raw_parts(
                self.buf.as_ptr() as *const u16,
                self.buf.len(),
            )
        }
    }

    /// Clear the buffer with a color
    pub fn clear_color(&mut self, color: Rgb565) {
        self.buf.fill(color);
    }

    /// Set a pixel directly
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgb565) {
        if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
            self.buf[(y * DISPLAY_WIDTH + x) as usize] = color;
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Rgb565) {
        let x_end = (x + w).min(DISPLAY_WIDTH);
        let y_end = (y + h).min(DISPLAY_HEIGHT);
        for py in y..y_end {
            for px in x..x_end {
                self.buf[(py * DISPLAY_WIDTH + px) as usize] = color;
            }
        }
    }

    /// Draw a horizontal line
    pub fn hline(&mut self, x: u32, y: u32, w: u32, color: Rgb565) {
        self.fill_rect(x, y, w, 1, color);
    }

    /// Draw a vertical line
    pub fn vline(&mut self, x: u32, y: u32, h: u32, color: Rgb565) {
        self.fill_rect(x, y, 1, h, color);
    }

    /// Get display width
    pub fn width(&self) -> u32 {
        DISPLAY_WIDTH
    }

    /// Get display height
    pub fn height(&self) -> u32 {
        DISPLAY_HEIGHT
    }
}

impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }
}

impl DrawTarget for FrameBuffer {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let x = coord.x;
            let y = coord.y;
            if x >= 0 && x < DISPLAY_WIDTH as i32 && y >= 0 && y < DISPLAY_HEIGHT as i32 {
                self.buf[(y as u32 * DISPLAY_WIDTH + x as u32) as usize] = color;
            }
        }
        Ok(())
    }
}
