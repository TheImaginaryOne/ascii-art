use rusttype::{Font, Scale};

/// Intensity divided as follows
///
/// lt t rt
/// l  m  r
/// lb b rt
#[derive(Clone, Debug, Default)]
pub struct Intensity {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub middle: f32,
}
impl Intensity {
    pub fn distance(&self, other: &Intensity) -> f32 {
        (self.left - other.left).abs()
            + (self.right - other.right).abs()
            + (self.top - other.top).abs()
            + (self.bottom - other.bottom).abs() * 3.
            + (self.middle - other.middle).abs()
    }
    pub fn apply<T: Fn(f32) -> f32>(&mut self, func: T) {
        self.left = func(self.left);
        self.right = func(self.right);
        self.top = func(self.top);
        self.bottom = func(self.bottom);
        self.middle = func(self.middle);
    }
}
pub type CharIntensities = Vec<(char, Intensity)>;

pub fn char_intensities(
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn distance() {
        let a = Intensity {
            left: 0.5,
            right: 0.5,
            top: 0.5,
            bottom: 0.5,
            middle: 0.5,
        };
        let b = Intensity {
            left: 0.,
            right: 1.,
            top: 0.,
            bottom: 1.,
            middle: 0.,
        };
        assert_eq!(a.distance(&b), 0.5 + 0.5 + 0.5 + 0.5 + 1.5);
    }
}
