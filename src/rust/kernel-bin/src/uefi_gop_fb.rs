//! Module for the UEFI frame buffer logger.

use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Write};
use core::{ptr, slice};
use font8x8::UnicodeFonts;
use kernel_lib::mutex::SimpleMutex;
use uefi::proto::console::gop::{
    FrameBuffer, GraphicsOutput, Mode, ModeInfo, PixelBitmask, PixelFormat,
};
use uefi::table::{Boot, SystemTable};
use uefi::{Completion, ResultExt};

const PREFERRED_HEIGHT: usize = 768;
const PREFERRED_WIDTH: usize = 1024;

/// Additional vertical space between lines
const LINE_SPACING: usize = 0;

/// Defines the framebuffer my kernel uses, that was initialized by the
/// Graphics Output Protocol (GOP) of UEFI. This code is heavily inspired
/// by the `bootloader` crate.
pub struct UefiGopFramebuffer<'a> {
    // Framebuffer object from UEFI
    framebuffer_obj: FrameBuffer<'a>,
    // Framebuffer slice (memory mapped I/O)
    framebuffer_slice: &'a mut [u8],
    // Graphics Mode used by UEFI framebuffer
    framebuffer_mode: ModeInfo,

    // current write position
    x_pos: usize,
    // current read position
    y_pos: usize,
}

impl<'a> UefiGopFramebuffer<'a> {
    pub fn new(table: &SystemTable<Boot>) -> Result<Arc<SimpleMutex<Self>>, ()> {
        let gop = table
            .boot_services()
            .locate_protocol::<GraphicsOutput>()
            .expect_success("failed to locate gop");
        let gop = unsafe { &mut *gop.get() };
        let mode = Self::choose_gop_mode(gop)?;

        gop.set_mode(&mode)
            .expect_success("Failed to apply the desired display mode");

        let framebuffer_mode = gop.current_mode_info();
        let mut framebuffer = gop.frame_buffer();
        let framebuffer_slice =
            unsafe { slice::from_raw_parts_mut(framebuffer.as_mut_ptr(), framebuffer.size()) };

        let mut obj = Self {
            framebuffer_obj: framebuffer,
            framebuffer_slice,
            framebuffer_mode,

            x_pos: 0,
            y_pos: 0,
        };
        obj.clear();

        log::debug!("UEFI Framebuffer initialized!");
        log::debug!(
            "Using UEFI GOP Mode: {}x{}, pixel_format={:?}",
            obj.width(),
            obj.height(),
            obj.pixel_format()
        );

        Ok(Arc::new(SimpleMutex::new(obj)))
    }

    // INTERNAL HELPERS

    /// Matches all available modes against the preferred window height and width.
    /// Returns the closest one matching it, if available.
    fn choose_gop_mode(gop: &GraphicsOutput) -> Result<Mode, ()> {
        /*let modes = gop.modes().map(Completion::unwrap);
        match (Some(PREFERRED_HEIGHT), Some(PREFERRED_WIDTH)) {
            (Some(height), Some(width)) => modes
                .filter(|m| {
                    let res = m.info().resolution();
                    res.1 >= height && res.0 >= width
                })
                .last(),
            (Some(height), None) => modes.filter(|m| m.info().resolution().1 >= height).last(),
            (None, Some(width)) => modes.filter(|m| m.info().resolution().0 >= width).last(),
            _ => None,
        }
        .ok_or(())*/
        gop.modes()
            .map(Completion::unwrap)
            .filter(|mode: &Mode| {
                let w = mode.info().resolution().0;
                let h = mode.info().resolution().1;
                w == PREFERRED_WIDTH && h == PREFERRED_HEIGHT
            })
            .next()
            .ok_or(())
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                if self.x_pos >= self.width() {
                    self.newline();
                }
                if self.y_pos >= (self.height() - 8) {
                    self.clear();
                }
                let rendered = font8x8::BASIC_FONTS
                    .get(c)
                    .expect("character not found in basic font");
                self.write_rendered_char(rendered);
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: [u8; 8]) {
        for (y, byte) in rendered_char.iter().enumerate() {
            for (x, bit) in (0..8).enumerate() {
                let alpha = if *byte & (1 << bit) == 0 { 0 } else { 255 };
                self.write_pixel(self.x_pos + x, self.y_pos + y, alpha);
            }
        }
        self.x_pos += 8;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.stride() + x;
        let color = match self.pixel_format() {
            PixelFormat::Rgb => [intensity, intensity, intensity, 0],
            PixelFormat::Bgr => [intensity, intensity, intensity, 0],
            _ => panic!("invalid pixel format"),
        };
        let bytes_per_pixel = self.bytes_per_pixel();
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer_slice[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer_slice[byte_offset]) };
    }

    fn newline(&mut self) {
        self.y_pos += 8 + LINE_SPACING;
        self.carriage_return()
    }

    pub fn add_vspace(&mut self, space: usize) {
        self.y_pos += space;
    }

    fn carriage_return(&mut self) {
        self.x_pos = 0;
    }

    // PUBLIC HELPERS

    /// Erases all text on the screen.
    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.framebuffer_slice.fill(0);
    }

    // GETTERS

    pub fn height(&self) -> usize {
        self.framebuffer_mode.resolution().1
    }

    pub fn width(&self) -> usize {
        self.framebuffer_mode.resolution().0
    }

    pub fn pixel_format(&self) -> PixelFormat {
        self.framebuffer_mode.pixel_format()
    }

    pub fn pixel_bitmask(&self) -> Option<PixelBitmask> {
        self.framebuffer_mode.pixel_bitmask()
    }

    /// See [`ModeInfo::stride`].
    pub fn stride(&self) -> usize {
        self.framebuffer_mode.stride()
    }

    pub fn bytes_per_pixel(&self) -> usize {
        4
    }
}

impl<'a> Write for UefiGopFramebuffer<'a> {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        string.chars().for_each(|c| self.write_char(c));
        Ok(())
    }
}

impl<'a> Debug for UefiGopFramebuffer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UefiGopFramebuffer")
            .field("framebuffer_mode", &self.framebuffer_mode)
            .field("width", &self.width())
            .field("height", &self.height())
            .field("x_pos", &self.x_pos)
            .field("y_pos", &self.y_pos)
            .finish()
    }
}
