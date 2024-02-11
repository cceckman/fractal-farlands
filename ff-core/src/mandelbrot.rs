/// Implementation of the Mandelbrot fractal,
/// parameterized on a numeric type.
use crate::numeric::Complex;

/// Evaluate an X-range of the Mandelbrot fractal.
///
/// The Y-coordinate is fixed for the entire call; all points are to be evaluated at the same Y
/// coordinate.
/// The call provides a list of X-coordinates to evaluate at.
///
/// The output buffer must be the same length as the list of X-coordinates.
///
fn eval_range<N>(iteration_limit: u32, y: &N, input_x: &[N], output: &mut [Option<u32>])
where
    N: MandelbrotNumber + Clone,
{
    for (x, out) in input_x.iter().zip(output.iter_mut()) {
        *out = mandelbrot(iteration_limit, x, y);
    }
}

mod mandelbrot_number {
    //! Trait bounds on the numbers that can be used for the Mandelbrot computation.
    //
    // Use the hack from https://github.com/rust-lang/rfcs/pull/1672#issuecomment-1405377983
    // to implement disjoint auto traits:
    //
    // - A default impl for everything that implements From<i8>
    // - A non-default impl for everything else

    use num::BigInt;

    use crate::numeric::AddMulSub;
    /// Type constraint for use in the Mandelbrot fractal.
    pub trait MandelbrotNumber: AddMulSub + PartialOrd + Clone {
        /// Returns the value representing zero in this type.
        fn zero() -> Self;

        /// Returns the value representing four in this type.
        fn four() -> Self;
    }

    impl<T, K> MandelbrotNumber for T
    where
        T: MandelbrotNumberHelper<K>,
    {
        fn zero() -> Self {
            <T as MandelbrotNumberHelper<K>>::zero()
        }

        fn four() -> Self {
            <T as MandelbrotNumberHelper<K>>::four()
        }
    }

    /// Helper to implement MandelbrotNumber: disjoint by virtue of getting different T parameters.
    trait MandelbrotNumberHelper<T> {
        fn zero() -> Self;
        fn four() -> Self;
    }

    impl<N> MandelbrotNumberHelper<N> for N
    where
        N: From<i8>,
    {
        fn zero() -> Self {
            0.into()
        }
        fn four() -> Self {
            4.into()
        }
    }

    impl<N, M> MandelbrotNumberHelper<N> for N
    where
        M: From<i8>,
        N: From<M>,
    {
        fn zero() -> Self {
            0.into().into()
        }
        fn four() -> Self {
            4.into().into()
        }
    }
}

use mandelbrot_number::MandelbrotNumber;

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
fn mandelbrot<N>(limit: u32, x: &N, y: &N) -> Option<u32>
where
    N: MandelbrotNumber,
{
    let c: Complex<N> = Complex {
        re: x.clone(),
        im: y.clone(),
    };
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
        assert_eq!(mandelbrot::<N>(5, &N::zero(), &N::zero()), None);
        mandelbrot::<N>(5, &N::four(), &N::zero()).unwrap();
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

    #[test]
    #[allow(non_snake_case)]
    fn test_BigRational() {
        tests::<num::BigRational>();
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
