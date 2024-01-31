/// Implementation of the Mandelbrot fractal,
/// parameterized on a numeric type.
use crate::{AddMulSub, Complex};
use num::BigRational;

/// Evaluator for the Mandelbrot fractal.
///
/// This trait allows evaluation and comparison of the Mandelbrot fractal
/// to be dispatched dynamically.
pub trait MandelbrotEval {
    /// Provides a descriptive name for this evaluator, e.g. `float32` or `posit16`.
    fn name(&self) -> &str;

    fn eval_range(
        iteration_limit: u32,
        y: &BigRational,
        input_x: &[BigRational],
        output: &mut [Option<u32>],
    );
}

/// Evaluate an X-range of the Mandelbrot fractal.
///
/// The Y-coordinate is fixed for the entire call; all points are to be evaluated at the same Y
/// coordinate.
/// The call provides a list of X-coordinates to evaluate at.
///
/// The output buffer must be the same length as the list of X-coordinates.
///
/// Both coordinates are provided as BigRational, and are therefore representing an exact (pixel)
/// coordinate; the implementation of N may lose precision when converting these to their internal
/// representation.
fn eval_range<N>(
    iteration_limit: u32,
    y: &BigRational,
    input_x: &[BigRational],
    output: &mut [Option<u32>],
) where
    N: MandelbrotNumber + crate::ApproximateFromBigRational,
{
    let y = N::approximate(y);
    for (x, out) in input_x.iter().zip(output.iter_mut()) {
        let x = N::approximate(x);
        *out = mandelbrot(iteration_limit, x, y.clone());
    }
}

/// Type constraint for use in the Mandelbrot fractal.
trait MandelbrotNumber: AddMulSub + PartialOrd + Clone {
    /// Returns the value representing zero in this form.
    fn zero() -> Self;

    /// Returns the value representing four in this form.
    fn four() -> Self;
}

impl<N> MandelbrotNumber for N
where
    N: AddMulSub + From<i8> + PartialOrd + Clone,
{
    fn zero() -> Self {
        0.into()
    }
    fn four() -> Self {
        4.into()
    }
}

/// Sample a coordinate in the Mandelbrot fractal.
///
/// Runs up to `limit` iterations of the Mandelbrot computation,
/// $$
/// z_n = 0
/// z_n+1 = (z_n)^2 + c
/// $$
/// where $c = x + y * j$ is a coordinate on the complex plane.
///
/// The return value indicates if `z` tends towards infinity;
/// specifically, it indicates the number of iterations after which
/// $|z| >= 2$.
fn mandelbrot<N>(limit: u32, x: N, y: N) -> Option<u32>
where
    N: MandelbrotNumber,
{
    let c: Complex<N> = Complex { re: x, im: y };
    let mut z: Complex<N> = Complex {
        re: N::zero(),
        im: N::zero(),
    };
    let four: N = N::four();
    for i in 0..=limit {
        let distance = z.re.clone() * z.re.clone() + z.im.clone() * z.im.clone();
        // The Mandelbrot "escape condition" is that the Cartesian distance from the zero point
        // of the complex plane (0 + 0i) is at least two.
        // Normally, that distance is sqrt(x^2+y^2) - but we can skip the square-root and avoid
        // a trait requirement by comparing d^2 to 2^2 instead:
        if distance >= four {
            return Some(i);
        }
        z = z.clone() * z + c.clone();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tests<N>()
    where
        N: MandelbrotNumber,
    {
        assert_eq!(mandelbrot::<N>(5, N::zero(), N::zero()), None);
        mandelbrot::<N>(5, N::four(), N::zero()).unwrap();
    }

    #[test]
    fn test_f32() {
        tests::<f32>();
    }

    #[test]
    fn test_f64() {
        tests::<f64>();
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_P32() {
        tests::<softposit::P32>();
    }

    // Our implementation above relies on from<i8>, so we need at least
    // 7 integer bits plus a sign bit.
    // and any number of other fraction bits.
    type Fix24 = fixed::FixedI32<fixed::types::extra::U24>;
    type Fix56 = fixed::FixedI64<fixed::types::extra::U56>;

    #[test]
    fn test_fix24() {
        tests::<Fix24>();
    }

    #[test]
    fn test_fix56() {
        tests::<Fix56>();
    }
}
