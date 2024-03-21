use crate::{Escape, EscapeVector, Size, Zero, ZeroVector};
use hsv;

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
            }) => mandelbrot_to_rgb(min, max, count, z_magnitude_squared),
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
fn mandelbrot_to_rgb(min: usize, max: usize, value: usize, escape: f64) -> image::Rgb<u8> {
    // hue is in range [0, 1]
    let denom = match (max - min) as i64 {
        0 => 1,
        v => v,
    };
    // Smooth Mandelbrot coloring from https://mrob.com/pub/muency/continuousdwell.html
    let offset: i64 = ((4.0f64.log2().log2() - escape.log2().log2()) * 360.0) as i64;
    let hue_numerator = (value - min) as i64;
    // H in range [0, 360]
    // https://stackoverflow.com/questions/31210357/is-there-a-modulus-not-remainder-function-operation
    let hue = (((hue_numerator * 360 + offset) / denom) % 360 + 360) % 360;
    let (r, g, b) = hsv::hsv_to_rgb(hue as f64, 1.0, 1.0);
    image::Rgb([r, g, b])
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

        let max_zero = data.iter().fold(usize::MIN, |max, v| match v {
            None => max,
            Some(Zero { count: _, zero }) => std::cmp::max(max, *zero),
        });

        // The number of iteration is very long-tailed, use the 90th percentile
        // for coloring instead of the max.
        let mut sort_data: Vec<usize> = data
            .iter()
            .cloned()
            .filter_map(|a| match a {
                None => None,
                Some(Zero { count, .. }) => Some(count),
            })
            .collect();
        let len = sort_data.len();
        let high_iters = if sort_data.len() > 0 {
            let (_, high_iters, _) = sort_data.select_nth_unstable((len as f64 * 0.9) as usize);
            *high_iters
        } else {
            10000000
        };

        let pixel_values = data.into_iter().map(|v| match v {
            None => image::Rgb([0, 0, 0]),
            Some(Zero { count, zero }) => newton_to_rgb(max_zero + 1, zero, high_iters, count),
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

fn newton_to_rgb(
    num_zeros: usize,
    which_zero: usize,
    max_iters: usize,
    iters: usize,
) -> image::Rgb<u8> {
    let (r, g, b) = hsv::hsv_to_rgb(
        (which_zero * 360) as f64 / num_zeros as f64,
        1.0,
        ((iters as f64) / (max_iters as f64)).min(1.0),
    );
    image::Rgb([r, g, b])
}
