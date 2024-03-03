use std::ops::{Add, Div, Mul, Sub};

use num::{BigInt, BigRational, Signed, ToPrimitive};

use crate::{masked_float::MaskedFloat, numeric::FromRational};

/// A numeric type that can can be used for the Mandelbrot fractal.
///
/// This trait identifies the necessary operations to compute a Mandelbrot image:
/// - Mapping from BigRational and division- to get the area bounds of the image
/// - Addition, subtraction, multiplication - to implement complex numbers and the Mandelbrot image
/// - Constants zero and four - for initializing the image (zero) and bounds-checking (four)
/// - Comparison - for bounds-checking
pub trait MandelbrotNumber:
    Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Clone
    + PartialOrd<Self>
    + FromRational
    + std::fmt::Debug
{
    // Provides this type's representation of zero.
    fn zero() -> Self;

    // Provides this type's representation of four.
    fn four() -> Self;
}

impl MandelbrotNumber for f32 {
    fn zero() -> Self {
        0f32
    }

    fn four() -> Self {
        4f32
    }
}

impl MandelbrotNumber for f64 {
    fn zero() -> Self {
        0f64
    }

    fn four() -> Self {
        4f64
    }
}

impl MandelbrotNumber for BigRational {
    fn zero() -> Self {
        BigRational::new(0.into(), 1.into())
    }
    fn four() -> Self {
        BigRational::new(4.into(), 1.into())
    }
}

impl<const E: usize, const F: usize> MandelbrotNumber for MaskedFloat<E, F> {
    fn zero() -> Self {
        MaskedFloat::<E, F>::new(0.0)
    }
    fn four() -> Self {
        MaskedFloat::<E, F>::new(4.0)
    }
}

/// Implementation of MandelbrotNumber for fixed-precision formats.
/// Needs at least 4 bits of integer part to allow "4" + sign.
macro_rules! impl_fixed {
    ($t:ty) => {
        impl MandelbrotNumber for $t {
            fn zero() -> Self {
                Self::unwrapped_from_num(0)
            }
            fn four() -> Self {
                Self::unwrapped_from_num(4)
            }
        }

        impl FromRational for $t {
            fn from_bigrational(value: &BigRational) -> Result<Self, String> {
                let zero: BigInt = 0.into();
                // To make it easier to reason about fractions & numbers-
                // Do everything as "positive", then negate.
                let negative = value.is_negative();
                let value = if negative { -value } else { value.clone() };

                let whole = value
                    .trunc()
                    .to_integer()
                    .to_i128()
                    .ok_or_else(|| format!("big-rational {} out of range", value.trunc()))?;
                let part = value.fract();
                const FRAC: u32 = <$t>::FRAC_NBITS;

                // Perform binary long-division on the fractional part;
                // get one more bit of precision than we have.
                let mut remainder = part.numer().clone();
                let mut fraction: i128 = 0;
                for i in 0..=FRAC {
                    remainder = remainder << 1;
                    let new_rem = &remainder - part.denom();
                    if new_rem >= zero {
                        remainder = new_rem;
                        fraction |= 1 << (FRAC - i);
                    }
                }
                // We kept an extra bit of precision so we can round.
                let fraction = if (fraction & 1) == 1 {
                    // Round up:
                    (fraction >> 1) | 1
                } else {
                    // Round down:
                    (fraction >> 1)
                };

                let v = <$t>::from_bits(
                    ((whole << FRAC) | fraction)
                        .try_into()
                        .map_err(|_| "truncated rational".to_string())?,
                );
                Ok(if negative { -v } else { v })
            }
        }
    };
}

impl_fixed!(fixed::types::I11F5);
impl_fixed!(fixed::types::I13F3);
impl_fixed!(fixed::types::I15F1);

// TODO: implement for posits. How?

#[cfg(test)]
mod tests {
    use super::*;
    use num::BigInt;

    #[test]
    fn test_fixed_i16f3() {
        type T = fixed::types::I13F3;

        let x0: T = T::zero();
        assert_eq!(x0, T::unwrapped_from_num(0));
        let x4: T = T::four();
        assert_eq!(x4, T::unwrapped_from_num(4));

        let x1p5: T = T::from_bigrational(&BigRational::new(3.into(), 2.into())).unwrap();
        assert_eq!(x1p5, T::unwrapped_from_num(1.5));

        let x11eigths: T = T::from_bigrational(&BigRational::new(11.into(), 8.into())).unwrap();
        assert_eq!(x11eigths, T::unwrapped_from_num(1.375));

        let neg11eigths: T =
            T::from_bigrational(&BigRational::new((-11).into(), 8.into())).unwrap();
        assert_eq!(neg11eigths, T::unwrapped_from_num(-1.375));
    }

    #[test]
    fn test_fixed_rounding() {
        type T = fixed::types::I15F1;

        let x0: T = T::zero();
        assert_eq!(x0, T::unwrapped_from_num(0));
        let x4: T = T::four();
        assert_eq!(x4, T::unwrapped_from_num(4));

        let x1p5: T = T::from_bigrational(&BigRational::new(3.into(), 2.into())).unwrap();
        assert_eq!(x1p5, T::unwrapped_from_num(1.5));

        // Needs to round, we only have a half-bit of precision.
        let x11eigths: T = T::from_bigrational(&BigRational::new(11.into(), 8.into())).unwrap();
        assert_eq!(x11eigths, T::unwrapped_from_num(1.5));
    }

    #[test]
    fn test_masked_float_small() {
        // Define small positive and negative BigRational values
        let center: BigInt = 0.into();
        let scale: BigInt = 5000.into();
        let small_positive = BigRational::new(center.clone() + 1, scale.clone());
        let small_negative = BigRational::new(center.clone() - 1, scale.clone());

        // Create MaskedFloat<3, 50> instances from those values
        let positive_mf: MaskedFloat<3, 50> =
            FromRational::from_bigrational(&small_positive).unwrap();
        let negative_mf: MaskedFloat<3, 50> =
            FromRational::from_bigrational(&small_negative).unwrap();

        // Assert that they are negations of each other
        assert_eq!(positive_mf.to_f64(), -negative_mf.to_f64());
    }
}
