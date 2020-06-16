use image::{Pixel, Rgb, RgbImage};
use std::io::Write;

use rusttype::{Font, Scale};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextWriteError {
    // TODO add an ImageResult.
    #[error("error while writing an image")]
    Image(#[from] image::error::ImageError),
    #[error("std::io error")]
    StdIo(#[from] std::io::Error),
}

pub trait TextWrite<T> {
    fn flush(&mut self) -> Result<(), T>;

    fn write_char(&mut self, c: char) -> Result<(), T>;

    fn write_newline(&mut self) -> Result<(), T>;
}

pub struct StdTextWriter<T: Write> {
    writer: T,
}
impl<T: Write> StdTextWriter<T> {
    pub fn new(writer: T) -> Self {
        Self { writer }
    }
}
impl<T: Write> TextWrite<TextWriteError> for StdTextWriter<T> {
    fn flush(&mut self) -> Result<(), TextWriteError> {
        self.writer.flush()?;
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), TextWriteError> {
        let mut b = [0; 4];
        let slice = c.encode_utf8(&mut b);
        self.writer.write(&slice.as_bytes())?;
        Ok(())
    }

    fn write_newline(&mut self) -> Result<(), TextWriteError> {
        self.writer.write(b"\n")?;
        Ok(())
    }
}

pub struct ImageOptions<'a> {
    pub font: Font<'a>,
    pub text_scale: Scale,
    pub width: u32,
    pub height: u32,
    pub line_height: f32,
    pub text_colour: Rgb<u8>,
    pub background_colour: Rgb<u8>,
}

pub struct ImageTextWriter<'a> {
    canvas: RgbImage,
    path: String,
    font: Font<'a>,
    text_scale: Scale,
    text_colour: Rgb<u8>,

    current_line: String,
    current_y: f32,
    ascent: f32,
    line_height: f32,
}
impl<'a> ImageTextWriter<'a> {
    pub fn new<S: Into<String>>(path: S, options: ImageOptions<'a>) -> Self {
        let (ascent, line_height, character_width) = {
            let v_metrics = options.font.v_metrics(options.text_scale);
            let width = options
                .font
                .glyph(' ')
                .scaled(options.text_scale)
                .h_metrics()
                .advance_width;
            (v_metrics.ascent, options.line_height, width)
        };
        let canvas_width = (options.width as f32 * character_width).round() as u32;
        let canvas_height = (options.height as f32 * line_height).round() as u32;

        let mut canvas = RgbImage::new(canvas_width, canvas_height);
        for i in 0..canvas_width {
            for j in 0..canvas_height {
                canvas.put_pixel(i, j, options.background_colour);
            }
        }

        Self {
            canvas,
            path: path.into(),
            font: options.font,
            text_scale: options.text_scale,
            text_colour: options.text_colour,
            current_line: String::new(),
            current_y: 0.,
            ascent,
            line_height,
        }
    }
    fn draw_current_line(&mut self) {
        let position = rusttype::point(0., self.current_y + self.ascent);

        let text = self
            .font
            .layout(&self.current_line, self.text_scale, position);

        for glyph in text {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                let canvas = &mut self.canvas;
                let text_colour = self.text_colour;
                glyph.draw(|x, y, c| {
                    let x = x as i32 + bounding_box.min.x;
                    let y = y as i32 + bounding_box.min.y;
                    if 0 <= x
                        && x as u32 <= canvas.width() - 1
                        && 0 <= y
                        && y as u32 <= canvas.height() - 1
                    {
                        let current_pixel = canvas.get_pixel(x as u32, y as u32);
                        let new_pixel = text_colour.map2(&current_pixel, |x, y| {
                            ((x as f32 * c + y as f32 * (1. - c)).round() as u32)
                                .min(255)
                                .max(0) as u8
                        });
                        canvas.put_pixel(x as u32, y as u32, new_pixel);
                    }
                });
            }
        }
    }
}
impl<'a> TextWrite<TextWriteError> for ImageTextWriter<'a> {
    fn flush(&mut self) -> Result<(), TextWriteError> {
        self.canvas.save(&std::path::Path::new(&self.path))?;
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), TextWriteError> {
        self.current_line.push(c);
        Ok(())
    }

    fn write_newline(&mut self) -> Result<(), TextWriteError> {
        self.draw_current_line();
        self.current_y += self.line_height;
        self.current_line.clear();
        Ok(())
    }
}
