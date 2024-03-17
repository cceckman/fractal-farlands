use rayon::prelude::*;
use std::ops::Range;

// Implementation of Newton's fractal for z^3-1
// TODO:
//   Parameterize to other functions
use crate::{masked_float::MaskedFloat, numeric::Complex, CommonParams};

pub use crate::number::MandelbrotNumber;
use crate::{Zero, ZeroVector};
use num::BigRational;

/// Function pointer for evaluating zeros
type EscapeFn = fn(&CommonParams, usize) -> Result<ZeroVector, String>;

const FUNCTIONS: &[(&'static str, EscapeFn)] = &[
    ("f32", evaluate_parallel_numeric::<f32>),
    ("f64", evaluate_parallel_numeric::<f64>),
    /*("P32", evaluate_parallel_numeric::<softposit::P32>),
    ("P16", evaluate_parallel_numeric::<softposit::P16>),
    ("P8", evaluate_parallel_numeric::<softposit::P8>),*/
    ("MaskedFloat<3,50>", evaluate_parallel_numeric::<MaskedFloat<3, 50>>),
    ("MaskedFloat<4,50>", evaluate_parallel_numeric::<MaskedFloat<4, 50>>),
    //("I11F5", evaluate_parallel_numeric::<fixed::types::I11F5>),
];

/// List the numeric formats that are valid for rendering.
pub fn formats() -> impl Iterator<Item = &'static str> {
    FUNCTIONS.iter().map(|(name, _)| *name)
}

pub fn compute(params: &CommonParams, iterations: usize) -> Result<ZeroVector, String> {
    let fmt = params.numeric.as_str();
    // Linear scan, we don't have that many options:
    for (candidate, computer) in FUNCTIONS.iter() {
        if *candidate == fmt {
            return computer(params, iterations);
        }
    }

    Err(format!("unknown numeric format {}", fmt))
}

fn evaluate_parallel_numeric<N>(
    params: &CommonParams,
    iterations: usize,
) -> Result<ZeroVector, String>
where
    N: MandelbrotNumber + Send + Sync,
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
        //.par_bridge()
        //.into_par_iter()
        .into_iter()
        .for_each(|(y, row_out)| {
            xs.iter().zip(row_out).for_each(|(x, out)| {
                *out = find_zero(x, &y, iterations);
            })
        });

    let mut zero_index: Vec<Complex<N>> = Vec::new();

    Ok(zeros
        .into_iter()
        .map(|x| match x {
            None => None,
            Some((z, iters)) => match zero_index.iter().position(|x| (*x).near(z.clone(), z.clone(), N::from_i32(512))) {
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
    N: MandelbrotNumber,
{
    let mut z: Complex<N> = Complex {
        re: x.clone(),
        im: y.clone(),
    };

    let zero: Complex<N> = Complex {
        re: N::zero(),
        im: N::zero(),
    };

    let one: Complex<N> = Complex {
        re: N::one(),
        im: N::zero(),
    };

    let three: Complex<N> = Complex {
        re: N::from_i32(3),
        im: N::zero(),
    };

    //let three: N = N::from_i32(3);
    //let six: N = N::from_i32(6);

    for i in 0..limit {
        //println!("Z[{}]: re: {:?} im: {:?}", i, z.re, z.im);
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
        //println!("FZ  re: {:?} im: {:?}", fz.re, fz.im);
        //
        // NOTE: I think this might be wrong...
        let fpz = three.clone() * z.clone() * z.clone();
        /*let fpz = Complex {
            re: three.clone() * z.re.clone() * z.re.clone()
                - six.clone() * z.im.clone() * z.im.clone(),
            im: six.clone() * z.re.clone() * z.re.clone()
                - three.clone() * z.im.clone() * z.im.clone(),
        };*/
        //println!("FPZ re: {:?} im: {:?}", fpz.re, fpz.im);
        let del = fz.clone() / fpz;
        //println!("Del re: {:?} im: {:?}", del.re, del.im);
        //
        // NOTE: This won't work in general. Should do a "near zero" method
        if fz.near(zero.clone(), z.clone(), N::from_i32(1024)) {
            //println!("Done");
            //println!("Done: Z[{}]: re: {:?} im: {:?}", i, z.re, z.im);
            return Some((z, i));
        }
        z = z - del;
    }
    //println!("Fail: Z[{}]: re: {:?} im: {:?}", limit, z.re, z.im);
    return None;
}
