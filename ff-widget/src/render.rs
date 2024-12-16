use ff_core::{Escape, EscapeVector, Zero, ZeroVector};
use js_sys::Uint8ClampedArray;

/// Render a Mandelbrot-like fractal into an RGBA bitmap buffer.
///
/// Each point (pixel) is rendered as black if None, or corresponding to its value if Some.
pub fn mandelbrot(data: EscapeVector) -> Uint8ClampedArray {
    // Find min/max iterations, so we can compute hue in that scale
    let (min, max) = data
        .iter()
        .fold((usize::MAX, usize::MIN), |(min, max), v| match v {
            None => (min, max),
            Some(Escape { count, .. }) => (std::cmp::min(*count, min), std::cmp::max(*count, max)),
        });

    let array_len = (data.len() * 4) as u32;
    let array = Uint8ClampedArray::new_with_length(array_len);
    array.fill(0, 0, array_len);

    for (i, v) in data.into_iter().enumerate() {
        match v {
            None => (),
            Some(Escape {
                count,
                z_magnitude_squared,
            }) => {
                let px = mandelbrot_to_rgb(min, max, count, z_magnitude_squared);
                for (j, c) in px.into_iter().enumerate() {
                    array.set_index((i * 4 + j) as u32, c);
                }
            }
        }
        // Set opacity to max on all pixels.
        array.set_index((i * 4 + 3) as u32, 255);
    }

    array
}

/// Convert a value within a range to an RGB value.
fn mandelbrot_to_rgb(min: usize, max: usize, value: usize, escape: f64) -> [u8; 3] {
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
    [r, g, b]
}

/// Render a Newton fractal into a bitmap buffer.
///
/// The `data` vector must be `size.x * size.y` entries long.
/// Each point (pixel) is rendered as black if None, or corresponding to its value if Some.
pub fn newton(data: ZeroVector) -> Uint8ClampedArray {
    let max_zero = data.iter().fold(usize::MIN, |max, v| match v {
        None => max,
        Some(Zero { count: _, zero }) => std::cmp::max(max, *zero),
    });

    // The number of iteration is very long-tailed, use the 90th percentile
    // for coloring instead of the max.
    let mut sort_data: Vec<usize> = data
        .iter()
        .cloned()
        .filter_map(|a| a.map(|Zero { count, .. }| count))
        .collect();
    let len = sort_data.len();
    let high_iters = if len > 3 {
        let (_, i, _) = sort_data.select_nth_unstable((len as f64 * 0.9) as usize);
        *i
    } else {
        100
    };

    let array_len = (data.len() * 4) as u32;
    let array = Uint8ClampedArray::new_with_length(array_len);
    array.fill(0, 0, array_len);

    for (i, v) in data.into_iter().enumerate() {
        match v {
            None => (),
            Some(Zero { count, zero }) => {
                let px = newton_to_rgb(max_zero + 1, zero, high_iters, count);
                for (j, c) in px.into_iter().enumerate() {
                    array.set_index((i * 4 + j) as u32, c);
                }
            }
        }
        // Set opacity to max on all pixels.
        array.set_index((i * 4 + 3) as u32, 255);
    }

    array
}

fn newton_to_rgb(num_zeros: usize, which_zero: usize, max_iters: usize, iters: usize) -> [u8; 3] {
    let (r, g, b) = hsv::hsv_to_rgb(
        (which_zero * 360) as f64 / num_zeros as f64,
        1.0,
        ((iters as f64) / (max_iters as f64)).min(1.0),
    );
    [r, g, b]
}
