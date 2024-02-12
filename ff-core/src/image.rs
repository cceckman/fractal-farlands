use crate::Size;

/// Settings for rendering a fractal into an image.
#[derive(Default)]
pub struct Renderer {}

impl Renderer {
    /// Render a Mandelbrot-like fractal into an image.
    ///
    /// The `data` vector must be `size.x * size.y` entries long.
    /// Each point (pixel) is rendered as black if None, or corresponding to its value if Some.
    pub fn render(
        &self,
        size: Size,
        data: Vec<Option<usize>>,
    ) -> Result<image::DynamicImage, String> {
        if data.len() != (size.x * size.y) {
            return Err(format!(
                "error: data size != width * height: {} != {} * {}",
                data.len(),
                size.x,
                size.y
            ));
        }

        // Find min/max iterations, so we can compute hue in that scale
        let (min, max) = data
            .iter()
            .fold((usize::MAX, usize::MIN), |(min, max), v| match v {
                None => (min, max),
                Some(v) => (std::cmp::min(*v, min), std::cmp::max(*v, max)),
            });

        let pixel_values = data.into_iter().map(|v| match v {
            None => image::Rgb([0, 0, 0]),
            Some(v) => value_to_rgb(min, max, v),
        });

        let mut img = image::ImageBuffer::<image::Rgb<u8>, _>::new(size.x as u32, size.y as u32);
        img.pixels_mut()
            .zip(pixel_values)
            .for_each(|(pixel, value)| {
                *pixel = value;
            });

        Ok(img.into())
    }
}

/// Convert a value within a range to an RGB value.
fn value_to_rgb(min: usize, max: usize, value: usize) -> image::Rgb<u8> {
    // hue is in range [0, 1]
    let denom = match (max - min) as i64 {
        0 => 1,
        v => v,
    };
    let hue = num::Rational64::new((value - min) as i64, denom);
    // H in range [0, 360]
    let hue = (hue * 360).to_integer();

    // Formulas from Wikipedia:
    //
    // Chroma:
    //  C = (1 - |2L - 1|) * S_l
    // But we're fully saturating L=0.5, S=1, so C = 1.

    //  H' = H / 60deg
    let hprime = hue / 60;
    //  X = C * (1 - |H' mod 2 - 1|)
    // C == 1, so we can reduce that term out.
    let x = 1 - ((hprime % 2) - 1).abs();

    //  m = L - C / 2
    // but with L=0.5 and C = 1, m == 0.
    // And we want to scale to 0, 255...so:
    let x = (x as u8).saturating_mul(255);
    let c = 255;
    let (r, g, b) = if hprime < 1 {
        (c, x, 0)
    } else if hprime < 2 {
        (x, c, 0)
    } else if hprime < 3 {
        (0, c, x)
    } else if hprime < 4 {
        (0, x, c)
    } else if hprime < 5 {
        (x, 0, c)
    } else {
        (c, 0, x)
    };

    image::Rgb([r, g, b])
}
