use crate::{Escape, EscapeVector, Size, Zero, ZeroVector};

/// Settings for rendering a fractal into an image.
#[derive(Default)]
pub struct Renderer {}

impl Renderer {
    /// Render a Mandelbrot-like fractal into an image.
    ///
    /// The `data` vector must be `size.x * size.y` entries long.
    /// Each point (pixel) is rendered as black if None, or corresponding to its value if Some.
    pub fn render(&self, size: Size, data: EscapeVector) -> Result<image::DynamicImage, String> {
        if data.len() != (size.width * size.height) {
            return Err(format!(
                "error: data size != width * height: {} != {} * {}",
                data.len(),
                size.width,
                size.height
            ));
        }

        // Find min/max iterations, so we can compute hue in that scale
        let (min, max) = data
            .iter()
            .fold((usize::MAX, usize::MIN), |(min, max), v| match v {
                None => (min, max),
                Some(Escape { count, .. }) => {
                    (std::cmp::min(*count, min), std::cmp::max(*count, max))
                }
            });

        let pixel_values = data.into_iter().map(|v| match v {
            None => image::Rgb([0, 0, 0]),
            Some(Escape {
                count,
                z_magnitude_squared,
            }) => value_to_rgb(min, max, count, z_magnitude_squared),
        });

        let mut img =
            image::ImageBuffer::<image::Rgb<u8>, _>::new(size.width as u32, size.height as u32);
        img.pixels_mut()
            .zip(pixel_values)
            .for_each(|(pixel, value)| {
                *pixel = value;
            });

        Ok(img.into())
    }
}

#[derive(Default)]
pub struct NewtonRenderer {}

impl NewtonRenderer {
    /// Render a Newton fractal into an image.
    ///
    /// The `data` vector must be `size.x * size.y` entries long.
    /// Each point (pixel) is rendered as black if None, or corresponding to its value if Some.
    pub fn render(&self, size: Size, data: ZeroVector) -> Result<image::DynamicImage, String> {
        if data.len() != (size.width * size.height) {
            return Err(format!(
                "error: data size != width * height: {} != {} * {}",
                data.len(),
                size.width,
                size.height
            ));
        }

        let max = data.iter().fold(usize::MIN, |max, v| match v {
            None => max,
            Some(Zero { count: _, zero }) => std::cmp::max(max, *zero),
        });

        let pixel_values = data.into_iter().map(|v| match v {
            None => image::Rgb([0, 0, 0]),
            Some(Zero { count: _, zero }) => value_to_rgb(0, max, zero, 4.0),
        });

        let mut img =
            image::ImageBuffer::<image::Rgb<u8>, _>::new(size.width as u32, size.height as u32);
        img.pixels_mut()
            .zip(pixel_values)
            .for_each(|(pixel, value)| {
                *pixel = value;
            });

        Ok(img.into())
    }
}

/// Convert a value within a range to an RGB value.
fn value_to_rgb(min: usize, max: usize, value: usize, escape: f64) -> image::Rgb<u8> {
    // hue is in range [0, 1]
    let denom = match (max - min) as i64 {
        0 => 1,
        v => v,
    };
    // Smooth Mandelbrot coloring from https://mrob.com/pub/muency/continuousdwell.html
    let offset: i64 =
        ((4.0f64.log2().log2() - escape.log2().log2()) * (360.0f64 / (denom as f64))) as i64;
    let hue_numerator = (value - min) as i64;
    // H in range [0, 360]
    let hue = ((hue_numerator * 360) / denom) + offset;

    // Formulas from Wikipedia:
    //
    // Chroma:
    //  C = (1 - |2L - 1|) * S_l
    // But we're fully saturating L=0.5, S=1, so C = 1.

    //  H' = H / 60deg
    let hprime = (hue as f64) / 60.0;
    //  X = C * (1 - |H' mod 2 - 1|)
    // C == 1, so we can reduce that term out.
    let x = 1.0 - ((hprime % 2.0) - 1.0).abs();

    //  m = L - C / 2
    // but with L=0.5 and C = 1, m == 0.
    // And we want to scale to 0, 255...so:
    let x = (x * 255.0) as u8;
    let c = 255;
    let (r, g, b) = if hprime < 1.0 {
        (c, x, 0)
    } else if hprime < 2.0 {
        (x, c, 0)
    } else if hprime < 3.0 {
        (0, c, x)
    } else if hprime < 4.0 {
        (0, x, c)
    } else if hprime < 5.0 {
        (x, 0, c)
    } else {
        (c, 0, x)
    };

    image::Rgb([r, g, b])
}
