use std::{
    mem::size_of,
    ops::{Add, Div, Mul, Sub},
};

use num::{BigInt, BigRational, Signed, ToPrimitive};

use crate::{masked_float::MaskedFloat, numeric::FromRational};

/// A numeric type that can can be used for the Mandelbrot fractal.
///
/// This trait identifies the necessary operations to compute a Mandelbrot image:
/// - Mapping from BigRational and division- to get the area bounds of the image
/// - Addition, subtraction, multiplication - to implement complex numbers and the Mandelbrot image
/// - Constants zero and four - for initializing the image (zero) and bounds-checking (four)
/// - Comparison - for bounds-checking
pub trait FractalNumber:
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
    // Provides a way to turn an int into this type.
    fn from_i32(i: i32) -> Self;

    // Provides a way to get a f64 from this type.
    fn to_f64(self) -> f64;
}

impl FractalNumber for f32 {
    fn to_f64(self) -> f64 {
        self.into()
    }

    fn from_i32(i: i32) -> Self {
        i as f32
    }
}

impl FractalNumber for f64 {
    fn to_f64(self) -> f64 {
        self
    }

    fn from_i32(i: i32) -> Self {
        i.into()
    }
}

impl FractalNumber for BigRational {
    fn to_f64(self) -> f64 {
        ToPrimitive::to_f64(&self).unwrap()
    }

    fn from_i32(i: i32) -> Self {
        BigRational::new(i.into(), 1.into())
    }
}

impl<const E: usize, const F: usize> FractalNumber for MaskedFloat<E, F> {
    fn to_f64(self) -> f64 {
        self.into()
    }

    fn from_i32(i: i32) -> Self {
        MaskedFloat::<E, F>::new(i.into())
    }
}

fn to_fixed<F>(value: &BigRational) -> Result<F, String>
where
    F: fixed::traits::Fixed + std::ops::Neg<Output = F>,
{
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

    // Perform binary long-division on the fractional part;
    // get one more bit of precision than we have.
    let mut remainder = part.numer().clone();
    let mut fraction: i128 = 0;
    for i in 0..=F::FRAC_NBITS {
        remainder = remainder << 1;
        let new_rem = &remainder - part.denom();
        if new_rem >= zero {
            remainder = new_rem;
            fraction |= 1 << (F::FRAC_NBITS - i);
        }
    }
    // We kept an extra bit of precision so we can round.
    let fraction = if (fraction & 1) == 1 {
        // Round up:
        (fraction >> 1) | 1
    } else {
        // Round down:
        fraction >> 1
    };

    let v = F::from_bits(
        ((whole << F::FRAC_NBITS) | fraction)
            .try_into()
            .map_err(|_| "truncated rational".to_string())?,
    );
    Ok(if negative { -v } else { v })
}

/// Implementation of MandelbrotNumber for fixed-precision formats.
/// Needs at least 4 bits of integer part to allow "4" + sign.
macro_rules! impl_fixed {
    ($t:ty) => {
        impl FractalNumber for $t {
            fn to_f64(self) -> f64 {
                self.into()
            }

            fn from_i32(i: i32) -> Self {
                Self::saturating_from_num(i)
            }
        }

        impl FromRational for $t {
            fn from_bigrational(value: &BigRational) -> Result<Self, String> {
                to_fixed::<$t>(value)
            }
        }
    };
}

impl_fixed!(fixed::types::I11F5);
impl_fixed!(fixed::types::I13F3);
impl_fixed!(fixed::types::I15F1);
impl_fixed!(fixed::types::I22F10);
impl_fixed!(fixed::types::I20F12);

impl FromRational for softposit::P32 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String>
    where
        Self: Sized,
    {
        // A quire value is either NaR or an integer multiple of the square of minPos...
        // The smallest positive posit value, minPos, is ... Every posit value is an integer multiple of minPos.

        // So: we can convert from a BigRational to an N-bit posit by:
        // - Rounding to a multiple of the square of minPos
        //      (Note: _square of_ minPos - tricky!)
        // - Converting that to a quire
        // - Converting the quire to a posit

        // The smallest positive posit value, minPos, is 2^(−4n+8):
        // const MINEXP: i32 = -4 * 32 + 8;
        // TODO: Consider memoizing these constants, they're global but can't be static initialized.
        // They're cheap to initialize if we do it right- by bit-shifting-
        // but they can't do constexpr.
        let minpos_squared: BigRational = BigRational::new(2.into(), 1.into()).pow(16 - 8 * 32);
        let quire = {
            const QUIRE_BYTE_COUNT: usize = 16 * 32 / 8; // 16n bits / 8 bits per byte
            const QUIRE_WORD_COUNT: usize = QUIRE_BYTE_COUNT / size_of::<u64>();
            let bytes = {
                // We get the quire from the quotient:
                let quotient = (r / minpos_squared).to_integer();
                let mut bytes: Vec<u8> = quotient.to_signed_bytes_le().into();
                // We need to sign-extend until we have exactly 16n bits.
                // LE format means we append to sign-extend
                let byte = if r.is_negative() { 0xff } else { 0x0 };
                bytes.resize(QUIRE_BYTE_COUNT, byte);
                bytes
            };
            // For whatever reason... the softposit appears to track words in reverse order?
            let words: Vec<u64> = bytes
                .as_slice()
                .chunks_exact(size_of::<u64>())
                .map(|chunk| {
                    let mut word = [0u8; 8];
                    word.copy_from_slice(chunk);
                    u64::from_le_bytes(word)
                })
                .rev()
                .collect();

            let mut quire_words = [0u64; QUIRE_WORD_COUNT];
            quire_words.copy_from_slice(&words);
            softposit::Q32::from_bits(quire_words)
        };

        Ok(quire.to_posit())
    }
}

impl FromRational for softposit::P16 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(softposit::P32::from_bigrational(r)?.into())
    }
}

impl FromRational for softposit::P8 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(softposit::P32::from_bigrational(r)?.into())
    }
}

macro_rules! impl_posit {
    ($t:ty) => {
        impl FractalNumber for $t {
            fn from_i32(i: i32) -> Self {
                <$t>::from_i32(i)
            }

            fn to_f64(self) -> f64 {
                self.into()
            }
        }
    };
}

impl_posit!(softposit::P32);
impl_posit!(softposit::P16);
impl_posit!(softposit::P8);

#[cfg(test)]
mod tests {
    use super::*;
    use num::BigInt;
    use softposit::P32;

    #[test]
    fn test_fixed_i16f3() {
        type T = fixed::types::I13F3;

        let x0: T = T::from_i32(0);
        assert_eq!(x0, T::unwrapped_from_num(0));
        let x4: T = T::from_i32(4);
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

        let x0: T = T::from_i32(0);
        assert_eq!(x0, T::unwrapped_from_num(0));
        let x4: T = T::from_i32(4);
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

    #[test]
    fn test_bigint_deserialize() {
        let v = BigInt::from(1i8).to_signed_bytes_le();
        assert_eq!(&v, &[1u8]);
        let v = BigInt::from(-1i8).to_signed_bytes_le();
        assert_eq!(&v, &[0xffu8]);
    }

    #[test]
    fn test_p32_constants() {
        const ZERO: P32 = P32::from_f32(0.0);
        const ONE: P32 = P32::from_f32(1.0);
        const NEG: P32 = P32::from_f32(-1.0);
        let zero = BigRational::new(0.into(), 1.into());
        let one = BigRational::new(1.into(), 1.into());
        let neg = BigRational::new((-1).into(), 1.into());
        assert_eq!(P32::from_bigrational(&zero).unwrap(), ZERO);
        assert_eq!(P32::from_bigrational(&one).unwrap(), ONE);
        assert_eq!(P32::from_bigrational(&neg).unwrap(), NEG);
    }

    #[test]
    fn test_p32_small() {
        const SMALL: P32 = P32::from_f32(1.0 / 16.0);
        let small = BigRational::new(1.into(), 16.into());
        assert_eq!(P32::from_bigrational(&small).unwrap(), SMALL);
    }

    #[test]
    fn test_headers() {
        // Get the various representations for section headers
        let four_thirds = BigRational::new(4.into(), 3.into());
        let f = f32::from_bigrational(&four_thirds).unwrap();
        print!("{:2.11}\n", f);

      //let p4 = P32::from_bigrational(&four_thirds).unwrap();
      //let p5 = P32::from_bigrational(&five_thirds).unwrap();
      //print!("{:2.99}\n{:2.99}\n", p4, p5);

        // 2147483647 is the max that LaTeX will take.
        // 3330078125

        let n = 6;
        for i in 1..=n {
            let r = BigRational::new(i.into(), (n + 1).into());
            let fixie : fixed::types::I23F9 = to_fixed(&r).unwrap();
            print!("{:2.11}\n", fixie);
        }
    }
}
