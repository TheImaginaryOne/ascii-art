//
use structopt::StructOpt;
use image::io::Reader;
use image::GenericImageView;
use image::imageops::FilterType;
use rusttype::{Font, Scale};

use std::path::Path;
use anyhow::Context;
use ordered_float::OrderedFloat;

#[derive(Debug, StructOpt)]
#[structopt(name = "image")]
struct Options {
    #[structopt(short = "i", long)]
    filename: String,
    #[structopt(short = "w", long)]
    output_width: u32,
    #[structopt(short = "f", long)]
    font_filename: String,
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
    let image = Reader::open(Path::new(&options.filename))
        .context("Failed to open file!")?
        .decode()?;
    let new_width = options.output_width;
    let new_height = (image.height() * new_width) / image.width() * 1 / 2; // todo aspect ratio??
    println!("{} {}", new_width, new_height);

    println!("Resizing image");
    let new_image = image::imageops::resize(&image, new_width, new_height, FilterType::Triangle);

    // TODO
    let path = std::path::Path::new(&options.font_filename);
    let data = std::fs::read(path).context("Failed to open the font file!")?;
    let font: Font = Font::try_from_vec(data).ok_or(anyhow::anyhow!("Failed to load font"))?;

    let intensities = char_intensities("#@%xoe-/;:.,".chars(), font)?;
    //println!("{:?}", intensities);
    asciify(new_width, new_height, new_image, intensities);
    Ok(())
}

type CharIntensities = Vec<(char, f32)>;

fn char_intensities(chars: impl IntoIterator<Item = char>, font: Font) -> anyhow::Result<CharIntensities> {
    let height: u32 = 24; // TODO fix
    let scale = Scale { x: height as f32, y: height as f32 };

    let mut maximum: f32 = 0.;
    let mut intensities: CharIntensities = Vec::new();
    for character in chars {
        let glyph = font.glyph(character);
        let positioned = glyph
            .scaled(scale)
            .positioned(rusttype::point(0., font.v_metrics(scale).ascent));

        
        let width = (positioned.position().x 
            + positioned.unpositioned().h_metrics().advance_width)
            .round() as u32;

        let mut coverage: f32 = 0.;
        positioned.draw(|_x, _y, v| {
            //let v = (2. * (v - 0.5) + 0.5).min(0.).max(1.);
            if v > 0.5 {
                coverage += 1.;
            }
        });

        let intensity = coverage / (width * height) as f32;
        intensities.push((character, intensity));
        maximum = maximum.max(intensity);
    }
    for pair in intensities.iter_mut() {
        pair.1 /= maximum;
        // TODO how to handle this case?
        if pair.1 == std::f32::NAN {
            anyhow::bail!("A character has an intensity of NAN!");
        }
    }
    // "a string".chars() does not have a space
    intensities.push((' ', 0.));
    intensities.sort_by(|a, b| OrderedFloat(a.1).cmp(&OrderedFloat(b.1)));
    Ok(intensities)
}
fn intensity(pixel: &image::Rgba<u8>) -> f32 {
    pixel[0] as f32 * 0.3 + pixel[1] as f32 * 0.59 + pixel[2] as f32 * 0.11
}
fn asciify(new_width: u32, new_height: u32, new_image: image::RgbaImage, char_intensities: CharIntensities) {
    let mut maximum: f32 = 0.;
    let mut minimum: f32 = std::f32::MAX;
    
    let mut intensities: Vec<Vec<f32>> = vec![vec![0.; new_height as usize]; new_width as usize];
    for j in 0..new_height {
        for i in 0..new_width {
            let pixel = new_image.get_pixel(i, j);
            let avg = intensity(pixel);
             
            intensities[i as usize][j as usize] = avg as f32;

            if maximum < avg {
                maximum = avg;
            }
            if minimum > avg {
                minimum = avg;
            }
        }
    }
    for j in 0..new_height {
        for i in 0..new_width {
            let normalised = 1. - (intensities[i as usize][j as usize] - minimum) / (maximum - minimum);
            let mut a = 2. * (normalised.powf(1.7) - 0.5) + 0.5;
            a = a.max(0.).min(1.0);
            let next_char = char_intensities
                .iter()
                .min_by_key(|(_, i)| OrderedFloat((i - a).abs()))
                .unwrap_or(&(' ', 0.)).0; // todo
            print!("{}", next_char);
        }
        println!();
    }
}
