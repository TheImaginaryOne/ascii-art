use crate::text_write::{TextWrite, TextWriteError};
use image::imageops::FilterType;
use ordered_float::OrderedFloat;

use crate::intensity::{CharIntensities, Intensity};

fn avg_intensity(image: &image::GrayImage, x1: u32, y1: u32, x2: u32, y2: u32) -> f32 {
    let mut s: u32 = 0;
    for i in x1..x2 + 1 {
        for j in y1..y2 + 1 {
            s += image.get_pixel(i, j).0[0] as u32;
        }
    }
    s as f32 / ((x2 + 1 - x1) * (y2 + 1 - y1)) as f32 / 256.
}

pub fn asciify(
    writer: &mut dyn TextWrite<TextWriteError>,
    new_width: u32,
    new_height: u32,
    image: image::DynamicImage,
    char_intensities: CharIntensities,
    contrast: f32,
    gamma: f32,
) -> anyhow::Result<()> {
    let new_image =
        image::imageops::resize(&image, new_width * 3, new_height * 3, FilterType::Triangle);
    let grayscale = image::imageops::grayscale(&new_image);

    let mut intensities: Vec<Intensity> =
        vec![Intensity::default(); (new_height * new_width) as usize];
    for j in 0..new_height {
        for i in 0..new_width {
            let int = &mut intensities[(i + j * new_width) as usize];
            // send help
            int.left = avg_intensity(&grayscale, 3 * i, 3 * j, 3 * i, 3 * j + 2);
            int.right = avg_intensity(&grayscale, 3 * i + 2, 3 * j, 3 * i + 2, 3 * j + 2);
            int.top = avg_intensity(&grayscale, 3 * i, 3 * j, 3 * i + 2, 3 * j);
            int.bottom = avg_intensity(&grayscale, 3 * i, 3 * j + 2, 3 * i + 2, 3 * j + 2);
            int.middle = avg_intensity(&grayscale, 3 * i + 1, 3 * j + 1, 3 * i + 1, 3 * j + 1);
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
            writer.write_char(next_char)?;
        }
        writer.write_newline()?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn image_intensity() {
        let image = image::GrayImage::from_raw(3, 3, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).unwrap();
        assert_eq!(avg_intensity(&image, 1, 1, 2, 2), 7. / 256.);
        assert_eq!(avg_intensity(&image, 1, 1, 1, 1), 5. / 256.);
        assert_eq!(avg_intensity(&image, 0, 0, 0, 2), 4. / 256.);
    }
}
