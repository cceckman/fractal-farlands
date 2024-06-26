use rayon::prelude::*;
use std::{ops::Range, panic::AssertUnwindSafe};

/// Implementation of the Mandelbrot fractal,
/// parameterized on a numeric type.
use crate::{masked_float::MaskedFloat, numeric::Complex, CancelContext, CommonParams};

pub use crate::number::FractalNumber;
use crate::{Escape, EscapeVector};
use num::BigRational;

/// Function pointer for evaluating escape counts
type EscapeFn = fn( &dyn CancelContext, &CommonParams, usize) -> Result<EscapeVector, String>;

/// Pointers, by numeric format name:
const FUNCTIONS: &[(&'static str, EscapeFn)] = &[
    ("f32", evaluate_parallel_numeric::<f32>),
    ("f64", evaluate_parallel_numeric::<f64>),
    ("P32", evaluate_parallel_numeric::<softposit::P32>),
    ("P16", evaluate_parallel_numeric::<softposit::P16>),
    ("P8", evaluate_parallel_numeric::<softposit::P8>),
    (
        "MaskedFloat<3,50>",
        evaluate_parallel_numeric::<MaskedFloat<3, 50>>,
    ),
    (
        "MaskedFloat<4,50>",
        evaluate_parallel_numeric::<MaskedFloat<4, 50>>,
    ),
    (
        "MaskedFloat<6,3>",
        evaluate_parallel_numeric::<MaskedFloat<6, 3>>,
    ),
    ("I11F5", evaluate_parallel_numeric::<fixed::types::I11F5>),
    // ("I13F3", evaluate_parallel_numeric::<fixed::types::I13F3>),
    // ("I15F1", evaluate_parallel_numeric::<fixed::types::I15F1>),
];

/// List the numeric formats that are valid for rendering.
pub fn formats() -> impl Iterator<Item = &'static str> {
    FUNCTIONS.iter().map(|(name, _)| *name)
}

/// Computes the escape values in the given window.
///
/// Under the hood, this uses Rayon's par_iter, so it's recommended to launch it from a Rayon
/// thread-pool.
pub fn compute(ctx: &dyn CancelContext, params: &CommonParams, iterations: usize) -> Result<EscapeVector, String> {
    let fmt = params.numeric.as_str();
    // Linear scan, we don't have that many options:
    for (candidate, computer) in FUNCTIONS.iter() {
        if *candidate == fmt {
            return computer(ctx, params, iterations);
        }
    }

    Err(format!("unknown numeric format {}", fmt))
}

fn evaluate_parallel_numeric<N>(
    ctx: &dyn CancelContext,
    params: &CommonParams,
    iterations: usize,
) -> Result<EscapeVector, String>
where
    N: FractalNumber + Send + Sync,
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
            if ctx.is_canceled() {
                return;
            }
            // Catch the unwind before it makes it out of the Rayon worker thread.
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                xs.iter().zip(row_out).for_each(|(x, out)| {
                    *out = escape(x, &y, iterations);
                })
            }));
            if result.is_err() {
                tracing::error!("caught panic during mandelbrot evaluation");
            }
        });
    if ctx.is_canceled() {
        Err("canceled".to_string())
    } else {

        Ok(output)
    }

}

#[inline]
fn escape<N>(x: &N, y: &N, limit: usize) -> Option<Escape>
where
    N: FractalNumber,
{
    let mut z: Complex<N> = Complex {
        re: N::from_i32(0),
        im: N::from_i32(0),
    };
    let four: N = N::from_i32(4);
    let coord = Complex {
        re: x.clone(),
        im: y.clone(),
    };

    for i in 0..limit {
        let sq = z.square();
        z = sq + coord.clone();

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
