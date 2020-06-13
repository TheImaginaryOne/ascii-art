use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;
use rusttype::{Font, Scale};
use structopt::StructOpt;

use anyhow::Context;
use ordered_float::OrderedFloat;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, StructOpt)]
#[structopt(name = "image")]
struct Options {
    #[structopt(short = "i", long)]
    input_filename: String,
    #[structopt(short = "w", long)]
    output_width: u32,
    #[structopt(short = "f", long)]
    font_filename: String,
    #[structopt(short, long)]
    output_filename: Option<String>,

    #[structopt(short, long, default_value = "1.0")]
    contrast: f32,
    #[structopt(short, long, default_value = "1.0")]
    gamma: f32,
}

fn main() {
    let options = Options::from_args();
    if let Err(e) = run(options) {
        println!("Aiyah, an error! {}", e);
        if let Some(c) = e.source() {
            println!("  [{}]", c);
        }
    }
}
fn run(options: Options) -> anyhow::Result<()> {
    let image = Reader::open(Path::new(&options.input_filename))
        .context("Failed to open file!")?
        .decode()?;
    let new_width = options.output_width;
    let new_height = (image.height() * new_width) / image.width() * 1 / 2; // todo aspect ratio??
    println!("{} {}", new_width, new_height);

    println!("Resizing image");
    let new_image =
        image::imageops::resize(&image, new_width * 3, new_height * 3, FilterType::Triangle);
    let grayscale = image::imageops::grayscale(&new_image);

    let stdout = std::io::stdout();

    let out_buffer: Box<dyn Write> = match options.output_filename {
        Some(o) => Box::new(File::create(Path::new(&o))?),
        None => Box::new(stdout),
    };
    // TODO
    let path = std::path::Path::new(&options.font_filename);
    let data = std::fs::read(path).context("Failed to open the font file!")?;
    let font: Font = Font::try_from_vec(data).ok_or(anyhow::anyhow!("Failed to load font"))?;

    let intensities = char_intensities((32u8..127u8).map(|x| x as char), font)?;
    asciify(
        out_buffer,
        new_width,
        new_height,
        grayscale,
        intensities,
        options.contrast,
        options.gamma,
    )?;

    Ok(())
}

/// Intensity divided as follows
///
/// lt t rt
/// l  m  r
/// lb b rt
#[derive(Clone, Debug, Default)]
struct Intensity {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    middle: f32,
}
impl Intensity {
    fn distance(&self, other: &Intensity) -> f32 {
        (self.left - other.left).abs()
            + (self.right - other.right).abs()
            + (self.top - other.top).abs()
            + (self.bottom - other.bottom).abs() * 3.
            + (self.middle - other.middle).abs()
    }
    fn apply<T: Fn(f32) -> f32>(&mut self, func: T) {
        self.left = func(self.left);
        self.right = func(self.right);
        self.top = func(self.top);
        self.bottom = func(self.bottom);
        self.middle = func(self.middle);
    }
}
type CharIntensities = Vec<(char, Intensity)>;

fn char_intensities(
    chars: impl IntoIterator<Item = char>,
    font: Font,
) -> anyhow::Result<CharIntensities> {
    let height: u32 = 36; // TODO fix

    // x = y means uniform scale
    let scale = Scale {
        x: height as f32,
        y: height as f32,
    };

    let mut maximum: f32 = 0.;
    let mut intensities: CharIntensities = Vec::new();

    for character in chars {
        let glyph = font.glyph(character);
        let positioned = glyph
            .scaled(scale)
            .positioned(rusttype::point(0., font.v_metrics(scale).ascent));

        let width = (positioned.position().x + positioned.unpositioned().h_metrics().advance_width)
            .round() as u32;

        let side_width = width * 1 / 3;
        let top_height = height * 1 / 3;

        if let Some(bounding_box) = positioned.pixel_bounding_box() {
            let mut intensity = Intensity::default();

            positioned.draw(|x, y, v| {
                //let v = (2. * (v - 0.5) + 0.5).min(0.).max(1.);
                let x = x as i32 + bounding_box.min.x;
                let y = y as i32 + bounding_box.min.y;
                let mut on_side = false;
                if x < side_width as i32 {
                    intensity.left += v;
                    on_side = true;
                }
                if x >= (width - side_width) as i32 {
                    intensity.right += v;
                    on_side = true;
                }
                if y < top_height as i32 {
                    intensity.top += v;
                    on_side = true;
                }
                if y >= (height - top_height) as i32 {
                    intensity.bottom += v;
                    on_side = true;
                }
                if !on_side {
                    intensity.middle += v;
                }
            });

            intensity.bottom /= (top_height * width) as f32;
            intensity.top /= (top_height * width) as f32;
            intensity.left /= (side_width * height) as f32;
            intensity.right /= (side_width * height) as f32;
            intensity.middle /= ((width - side_width * 2) * (height - top_height * 2)) as f32;

            maximum = maximum
                .max(intensity.bottom)
                .max(intensity.top)
                .max(intensity.left)
                .max(intensity.right)
                .max(intensity.middle);

            intensities.push((character, intensity));
        }
    }
    // normalise
    for pair in intensities.iter_mut() {
        pair.1.bottom /= maximum;
        pair.1.top /= maximum;
        pair.1.left /= maximum;
        pair.1.right /= maximum;
        pair.1.middle /= maximum;

        // pixels with more intensity are darker -- 0. = darkest, 1. = lightest.
        pair.1.apply(|x| 1. - x);
    }
    // "a string".chars() does not have a space
    intensities.push((
        ' ',
        Intensity {
            bottom: 1.,
            top: 1.,
            left: 1.,
            right: 1.,
            middle: 1.,
        },
    ));
    Ok(intensities)
}

fn avg_intensity(image: &image::GrayImage, x1: u32, y1: u32, x2: u32, y2: u32) -> f32 {
    let mut s: u32 = 0;
    for i in x1..x2 + 1 {
        for j in y1..y2 + 1 {
            s += image.get_pixel(i, j).0[0] as u32;
        }
    }
    s as f32 / ((x2 + 1 - x1) * (y2 + 1 - y1)) as f32 / 256.
}

fn asciify(
    mut buffer: Box<dyn Write>,
    new_width: u32,
    new_height: u32,
    new_image: image::GrayImage,
    char_intensities: CharIntensities,
    contrast: f32,
    gamma: f32,
) -> anyhow::Result<()> {
    let mut intensities: Vec<Intensity> =
        vec![Intensity::default(); (new_height * new_width) as usize];
    for j in 0..new_height {
        for i in 0..new_width {
            let int = &mut intensities[(i + j * new_width) as usize];
            // send help
            int.left = avg_intensity(&new_image, 3 * i, 3 * j, 3 * i, 3 * j + 2);
            int.right = avg_intensity(&new_image, 3 * i + 2, 3 * j, 3 * i + 2, 3 * j + 2);
            int.top = avg_intensity(&new_image, 3 * i, 3 * j, 3 * i + 2, 3 * j);
            int.bottom = avg_intensity(&new_image, 3 * i, 3 * j + 2, 3 * i + 2, 3 * j + 2);
            int.middle = avg_intensity(&new_image, 3 * i + 1, 3 * j + 1, 3 * i + 1, 3 * j + 1);
        }
    }
    for j in 0..new_height {
        for i in 0..new_width {
            let a = &mut intensities[(i + j * new_width) as usize];
            a.apply(|x: f32| {
                let f: f32 = contrast * (x.powf(gamma) - 0.5) + 0.5;
                f.max(0.0).min(1.0)
            });
            let next_char = char_intensities
                .iter()
                .min_by_key(|(_, x)| OrderedFloat(a.distance(&x)))
                .unwrap_or(&(' ', Intensity::default()))
                .0; // todo
            buffer.write(&[next_char as u8])?;
        }
        buffer.write(b"\n")?;
    }
    Ok(())
}
