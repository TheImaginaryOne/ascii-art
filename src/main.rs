//
use structopt::StructOpt;
use image::io::Reader;
use image::GenericImageView;
use image::imageops::FilterType;

use std::path::Path;
use anyhow::Context;

#[derive(Debug, StructOpt)]
#[structopt(name = "image")]
struct Options {
    #[structopt(short, long)]
    filename: String,
    #[structopt(short = "w", long)]
    output_width: u32
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
    
    asciify(new_width, new_height, new_image);
    Ok(())
}
fn intensity(pixel: &image::Rgba<u8>) -> f32 {
    pixel[0] as f32 * 0.3 + pixel[1] as f32 * 0.59 + pixel[2] as f32 * 0.11
}
fn asciify(new_width: u32, new_height: u32, new_image: image::RgbaImage) {
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
    let chars: Vec<char> = vec![' ','.',',',':',';','x','%','#'];
    for j in 0..new_height {
        for i in 0..new_width {
            let normalised = 1. - (intensities[i as usize][j as usize] - minimum) / (maximum - minimum);
            let mut a = 1.1 * (normalised.powf(0.8) - 0.5) + 0.5;
            a = a.max(0.).min(1.0);
            let mut t = (8. * a).floor() as u32;
            if t == 8 {
                t -= 1;
            }
            print!("{}", chars[t as usize]);
        }
        println!();
    }
}
