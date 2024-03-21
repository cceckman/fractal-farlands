use fixed::types::{I11F5, I20F12, I22F10};
use rayon::prelude::*;
use std::{ops::Range, panic::AssertUnwindSafe};

// Implementation of Newton's fractal for z^3-1
// TODO:
//   Parameterize to other functions
use crate::{masked_float::MaskedFloat, numeric::Complex, CancelContext, CommonParams};

pub use crate::number::FractalNumber;
use crate::{Zero, ZeroVector};
use num::BigRational;

/// Function pointer for evaluating zeros
type EscapeFn = fn(&dyn CancelContext, &CommonParams, usize) -> Result<ZeroVector, String>;

const FUNCTIONS: &[(&'static str, EscapeFn)] = &[
    ("f32", evaluate_parallel_numeric::<f32>),
    ("f64", evaluate_parallel_numeric::<f64>),
    ("P32", evaluate_parallel_numeric::<softposit::P32>),
    ("P16", evaluate_parallel_numeric::<softposit::P16>),
    // P8 and MaskedFloat<3,50> don't produce interesting images, mostly fail to converge.
    //("P8", evaluate_parallel_numeric::<softposit::P8>),
    //("MaskedFloat<3,50>", evaluate_parallel_numeric::<MaskedFloat<3, 50>>),
    (
        "MaskedFloat<4,50>",
        evaluate_parallel_numeric::<MaskedFloat<4, 50>>,
    ),
    ("I22F10", evaluate_parallel_numeric::<I22F10>),
    ("I20F12", evaluate_parallel_numeric::<I20F12>),
    ("I11F5", evaluate_parallel_numeric::<I11F5>),
];

/// List the numeric formats that are valid for rendering.
pub fn formats() -> impl Iterator<Item = &'static str> {
    FUNCTIONS.iter().map(|(name, _)| *name)
}

pub fn compute(ctx: &dyn CancelContext, params: &CommonParams, iterations: usize) -> Result<ZeroVector, String> {
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
) -> Result<ZeroVector, String>
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
    let mut zeros: Vec<Option<(Complex<N>, usize)>> = Vec::new();
    zeros.resize(size.width * size.height, None);

    let out_rows = zeros.chunks_mut(size.width);
    ys.into_iter()
        .zip(out_rows)
        .par_bridge()
        .into_par_iter()
        .for_each(|(y, row_out)| {
            if ctx.is_canceled() {
                return
            }

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                xs.iter().zip(row_out).for_each(|(x, out)| {
                    *out = find_zero(x, &y, iterations);
                })
            }));
            if result.is_err() {
                tracing::error!("caught panic during mandelbrot evaluation");
            }
        });

    let mut zero_index: Vec<Complex<N>> = Vec::new();

    if ctx.is_canceled() {
        return Err("canceled".to_string())
    }

    Ok(zeros
        .into_iter()
        .map(|x| match x {
            None => None,
            Some((z, iters)) => match zero_index
                .iter()
                .position(|x| (*x).near(z.clone(), z.clone(), N::from_i32(512)))
            {
                None => {
                    let nz = zero_index.len();
                    zero_index.push(z);
                    Some(Zero {
                        count: iters,
                        zero: nz,
                    })
                }
                Some(n) => Some(Zero {
                    count: iters,
                    zero: n,
                }),
            },
        })
        .collect())
}

#[inline]
fn find_zero<N>(x: &N, y: &N, limit: usize) -> Option<(Complex<N>, usize)>
where
    N: FractalNumber,
{
    let mut z: Complex<N> = Complex {
        re: x.clone(),
        im: y.clone(),
    };

    let zero: Complex<N> = Complex {
        re: N::from_i32(0),
        im: N::from_i32(0),
    };

    let one: Complex<N> = Complex {
        re: N::from_i32(1),
        im: N::from_i32(0),
    };

    let three: Complex<N> = Complex {
        re: N::from_i32(3),
        im: N::from_i32(0),
    };

    for i in 0..limit {
        // For the z^3-1 Newton's fractal, first, check if the value is zero at the
        // current position--if so, we're done.
        //
        // Otherwise, the next value is equal to:
        // x_1 = x_0 - f(x)/f'(x)
        //
        // For f(x)=x^3-1, f'(x)=3x^2
        //
        // TODO: The function and its derivative could come in as lambdas.
        let fz = z.clone() * z.clone() * z.clone() - one.clone();
        let fpz = three.clone() * z.clone() * z.clone();
        if fpz.re.clone() * fpz.re.clone() + fpz.im.clone() * fpz.im.clone() == N::from_i32(0) {
            return None;
        }
        let del = fz.clone() / fpz;
        if fz.near(zero.clone(), z.clone(), N::from_i32(1024)) {
            return Some((z, i));
        }
        z = z - del;
    }
    //println!("Fail: Z[{}]: re: {:?} im: {:?}", limit, z.re, z.im);
    return None;
}
