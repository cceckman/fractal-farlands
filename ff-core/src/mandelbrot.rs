use rayon::prelude::*;
use std::ops::{Add, Mul, Range};

/// Implementation of the Mandelbrot fractal,
/// parameterized on a numeric type.
use crate::{masked_float::MaskedFloat, numeric::Complex, CommonParams};

mod number;
use crate::{Escape, EscapeVector};
use num::BigRational;
pub use number::MandelbrotNumber;

/// Evaluate a mandelbrot fractal according to the params,
/// using any parallelism available in the provided pool.
pub fn evaluate_parallel(params: &CommonParams, iterations: usize) -> Result<EscapeVector, String> {
    let computer: fn(&CommonParams, usize) -> Result<EscapeVector, String> =
        match params.numeric.as_str() {
            "f32" => evaluate_parallel_numeric::<f32>,
            "f64" => evaluate_parallel_numeric::<f64>,
            "MaskedFloat<3,50>" => evaluate_parallel_numeric::<MaskedFloat<3, 50>>,
            "MaskedFloat<4,50>" => evaluate_parallel_numeric::<MaskedFloat<4, 50>>,
            "I11F5" => evaluate_parallel_numeric::<fixed::types::I11F5>,
            "I13F3" => evaluate_parallel_numeric::<fixed::types::I13F3>,
            "I15F1" => evaluate_parallel_numeric::<fixed::types::I15F1>,
            _ => {
                return Err(format!(
                    "unknown numeric format {}",
                    params.numeric.as_str()
                ))
            }
        };
    computer(params, iterations)
}

fn evaluate_parallel_numeric<N>(
    params: &CommonParams,
    iterations: usize,
) -> Result<EscapeVector, String>
where
    N: MandelbrotNumber + Send + Sync,
    for<'a> &'a N: Mul<Output = N>,
    for<'a> N: Add<&'a N, Output = N>,
{
    let size = params.size;
    // Create the X and Y ranges up-front:
    let make_range = |r: &Range<BigRational>, steps: usize| -> Result<Vec<N>, String> {
        let step = (&r.end - &r.start) / BigRational::new(steps.into(), 1.into());
        let mut results = Vec::with_capacity(steps);
        let mut next = r.start.clone();
        for _ in 0..steps {
            let converted = N::from_bigrational(&next)?;
            results.push(converted);
            next += &step;
        }
        Ok(results)
    };
    let xs = make_range(&params.x, size.width)?;
    let ys = make_range(&params.y, size.height)?;
    let mut output: EscapeVector = Vec::new();
    output.resize(size.width * size.height, None);

    let out_rows = output.chunks_mut(size.width);
    ys.into_iter()
        .zip(out_rows)
        .par_bridge()
        .into_par_iter()
        .for_each(|(y, row_out)| {
            xs.iter().zip(row_out).for_each(|(x, out)| {
                *out = escape(x, &y, iterations);
            })
        });

    Ok(output)
}

#[inline]
fn escape<N>(x: &N, y: &N, limit: usize) -> Option<Escape>
where
    N: MandelbrotNumber,
    for<'a> &'a N: Mul<Output = N>,
    for<'a> N: Add<&'a N, Output = N>,
{
    let mut z: Complex<N> = Complex {
        re: N::zero(),
        im: N::zero(),
    };
    let four: N = N::four();
    let coord = Complex {
        re: x.clone(),
        im: y.clone(),
    };

    for i in 0..limit {
        let sq = z.square();
        z = sq + &coord;

        let z_magnitude_squared = z.re.clone() * z.re.clone() + z.im.clone() * z.im.clone();

        // The Mandelbrot "escape condition" is that the Cartesian distance from the zero point
        // of the complex plane (0 + 0i) is at least two.
        // Normally, that distance is sqrt(x^2+y^2) - but we can skip the square-root and avoid
        // a trait requirement by comparing d^2 to 2^2 instead:
        if z_magnitude_squared >= four {
            return Some(Escape {
                count: i,
                z_magnitude_squared: z_magnitude_squared.to_f64(),
            });
        }
    }
    return None;
}
