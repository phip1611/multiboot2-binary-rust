//! Module for the UEFI frame buffer logger.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter, Write};
use core::{ptr, slice};
use kernel_lib::fakelock::FakeLock;
use noto_sans_mono_bitmap::{get_bitmap, BitmapChar, BitmapHeight, FontWeight};
use uefi::proto::console::gop::{
    FrameBuffer, GraphicsOutput, Mode, ModeInfo, PixelBitmask, PixelFormat,
};
use uefi::table::{Boot, SystemTable};
use uefi::{Completion, ResultExt};

const PREFERRED_HEIGHT: usize = 768;
const PREFERRED_WIDTH: usize = 1024;

/// Additional vertical space between lines in px.
const LINE_SPACING: usize = 2;

pub type RGB = (u8, u8, u8);

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

    /// current write position
    x_pos: usize,
    /// current read position
    y_pos: usize,

    /// Current RGB color for font.
    color: RGB,
    /// Current font size.
    bitmap_font_height: usize,
}

impl<'a> UefiGopFramebuffer<'a> {
    /// Default color is white font on black ground.
    const DEFAULT_FONT_COLOR: RGB = (255, 255, 255);

    /// Default font size is 16px.
    const DEFAULT_FONT_SIZE: usize = 18;

    pub fn new(table: &SystemTable<Boot>) -> Result<Arc<FakeLock<Self>>, ()> {
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

            color: Self::DEFAULT_FONT_COLOR,
            bitmap_font_height: Self::DEFAULT_FONT_SIZE,
        };
        obj.clear();

        log::debug!("UEFI Framebuffer initialized!");
        log::debug!(
            "Using UEFI GOP Mode: {}x{}, pixel_format={:?}",
            obj.width(),
            obj.height(),
            obj.pixel_format()
        );

        Ok(Arc::new(FakeLock::new(obj)))
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
            // also covered by bitmap font
            /*' ' => {
                self.x_pos += 10;
                return;
            }*/
            c => {
                if self.x_pos + self.bitmap_font_height >= self.width() {
                    self.newline();
                }
                if self.y_pos >= (self.height() - self.bitmap_font_height - 5) {
                    self.clear();
                }
                let get_bitmap_fn = |c| get_bitmap(c, FontWeight::Regular, BitmapHeight::Size18);
                let bitmap = get_bitmap_fn(c).unwrap_or(get_bitmap_fn(' ').unwrap());
                self.write_rendered_char(bitmap);
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: BitmapChar) {
        for (row_i, row) in rendered_char.bitmap().iter().enumerate() {
            for (col_i, opacity) in row.iter().enumerate() {
                let x_pos = self.x_pos + col_i;
                let y_pos = self.y_pos + row_i;
                let rgb = (*opacity as u8, *opacity as u8, *opacity as u8);
                self.write_pixel(x_pos, y_pos, rgb);
            }
        }

        self.x_pos += rendered_char.width();
    }

    fn write_pixel(&mut self, x: usize, y: usize, rgb: RGB) {
        //assert!(x <= self.width(), "width exceeded");
        //assert!(y <= self.height(), "height exceeded!");

        let pixel_offset = y * self.stride() + x;
        let color = match self.pixel_format() {
            PixelFormat::Rgb => [rgb.0, rgb.1, rgb.2, 0],
            PixelFormat::Bgr => [rgb.2, rgb.1, rgb.0, 0],
            _ => panic!("invalid pixel format"),
        };
        let bytes_per_pixel = self.bytes_per_pixel();
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer_slice[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer_slice[byte_offset]) };
    }

    fn newline(&mut self) {
        // self.font_size already includes vertical padding
        self.y_pos += self.bitmap_font_height + LINE_SPACING;
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
