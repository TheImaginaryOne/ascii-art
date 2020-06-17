mod ascii_generation;
mod intensity;
mod text_write;

use image::io::Reader;
use image::GenericImageView;
use rusttype::Font;
use structopt::{clap::arg_enum, StructOpt};

use anyhow::Context;
use std::fs::File;
use std::path::Path;

use ascii_generation::asciify;
use intensity::char_intensities;
use text_write::{ImageOptions, ImageTextWriter, StdTextWriter, TextWrite, TextWriteError};

arg_enum! {
    #[derive(Debug)]
    enum OutputType {
        Stdout,
        Text,
        Png,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "image")]
struct Options {
    #[structopt(short = "i", long)]
    input_filename: String,
    #[structopt(short = "w", long)]
    output_width: u32,
    #[structopt(short = "f", long)]
    font_filename: Option<String>,
    // the kebaba case is very important
    #[structopt(
        short,
        long,
        required_if("output-type", "png"),
        required_if("output-type", "text")
    )]
    output_filename: Option<String>,
    #[structopt(short = "t", long, possible_values = &OutputType::variants(), case_insensitive = true)]
    output_type: OutputType,

    #[structopt(short = "s", long, default_value = "12.")]
    font_size: f32,
    #[structopt(short = "a", long)]
    font_aspect: Option<f32>,
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
    let mut new_height = (image.height() * new_width) / image.width();
    println!("{} {}", new_width, new_height);
    let stdout = std::io::stdout();

    // TODO
    let font_data = if let Some(font_filename) = &options.font_filename {
        let path = std::path::Path::new(font_filename);
        std::fs::read(path).context("Failed to open the font file!")?
    } else {
        include_bytes!("font/SourceCodePro-Regular.otf").to_vec()
    };
    let font: Font = Font::try_from_vec(font_data).ok_or(anyhow::anyhow!("Failed to load font"))?;

    let font_aspect = if let Some(a) = options.font_aspect {
        a
    } else {
        let scale = rusttype::Scale { x: 12., y: 12. };
        let width = font.glyph(' ').scaled(scale).h_metrics().advance_width;
        width / 12.
    };
    new_height = (new_height as f32 * font_aspect) as u32;
    let mut output_writer: Box<dyn TextWrite<TextWriteError>> = match options.output_type {
        OutputType::Text => Box::new(StdTextWriter::new(File::create(Path::new(
            &options.output_filename.unwrap(),
        ))?)),
        OutputType::Stdout => Box::new(StdTextWriter::new(stdout)),
        OutputType::Png => {
            let scale = rusttype::Scale {
                x: options.font_size,
                y: options.font_size,
            };
            Box::new(ImageTextWriter::new(
                options.output_filename.unwrap(),
                ImageOptions {
                    font: font.clone(),
                    text_scale: scale,
                    width: new_width,
                    height: new_height,
                    line_height: scale.y * 1.,
                    // TODO customise
                    text_colour: image::Rgb::from([235, 88, 52]),
                    background_colour: image::Rgb::from([250, 250, 250]),
                },
            ))
        }
    };
    let intensities = char_intensities((32u8..127u8).map(|x| x as char), font)?;
    asciify(
        output_writer.as_mut(),
        new_width,
        new_height,
        image,
        intensities,
        options.contrast,
        options.gamma,
    )?;
    // in case
    output_writer.flush()?;

    Ok(())
}
