mod ascii_generation;
mod intensity;
mod text_write;

use image::io::Reader;
use image::GenericImageView;
use rusttype::Font;
use structopt::StructOpt;

use anyhow::Context;
use std::fs::File;
use std::path::Path;

use ascii_generation::asciify;
use intensity::char_intensities;
use text_write::{TextWrite, StdTextWriter, TextWriteError};

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
    let stdout = std::io::stdout();

    let mut out_buffer: Box<dyn TextWrite<TextWriteError>> = match options.output_filename {
        Some(o) => Box::new(StdTextWriter::new(File::create(Path::new(&o))?)),
        None => Box::new(StdTextWriter::new(stdout)),
    };
    // TODO
    let path = std::path::Path::new(&options.font_filename);
    let data = std::fs::read(path).context("Failed to open the font file!")?;
    let font: Font = Font::try_from_vec(data).ok_or(anyhow::anyhow!("Failed to load font"))?;

    let intensities = char_intensities((32u8..127u8).map(|x| x as char), font)?;
    asciify(
        out_buffer.as_mut(),
        new_width,
        new_height,
        image,
        intensities,
        options.contrast,
        options.gamma,
    )?;
    // in case
    out_buffer.flush()?;

    Ok(())
}
