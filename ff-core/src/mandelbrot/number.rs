use std::ops::{Add, Div, Mul, Sub};

use num::{BigRational, ToPrimitive};

use crate::masked_float::MaskedFloat;

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
{
    // Provides this type's representation of zero.
    fn zero() -> Self;

    // Provides this type's representation of four.
    fn four() -> Self;

    /// Converts from BigRational.
    ///
    /// This is provided as a distinct method because we can't expect `From<BigRational>`
    /// on foreign types.
    fn from_bigrational(value: &BigRational) -> Option<Self>;
}

impl MandelbrotNumber for f32 {
    fn zero() -> Self {
        0f32
    }

    fn four() -> Self {
        4f32
    }

    fn from_bigrational(value: &BigRational) -> Option<Self> {
        value.to_f32()
    }
}

impl MandelbrotNumber for f64 {
    fn zero() -> Self {
        0f64
    }

    fn four() -> Self {
        4f64
    }

    fn from_bigrational(value: &BigRational) -> Option<Self> {
        value.to_f64()
    }
}

impl MandelbrotNumber for BigRational {
    fn zero() -> Self {
        BigRational::new(0.into(), 1.into())
    }
    fn four() -> Self {
        BigRational::new(4.into(), 1.into())
    }
    fn from_bigrational(value: &BigRational) -> Option<Self> {
        Some(value.to_owned())
    }
}

impl<const E: usize, const F: usize> MandelbrotNumber for MaskedFloat<E, F> {
    fn zero() -> Self {
        MaskedFloat::<E, F>::new(0.0)
    }
    fn four() -> Self {
        MaskedFloat::<E, F>::new(4.0)
    }
    fn from_bigrational(value: &BigRational) -> Option<Self> {
        Some(MaskedFloat::<E, F>::new(value.to_f64()?))
    }
}

// TODO: Implement for FixedI32, FixedI16, etc etc.
// BigRational can be broken into its integer and fractional parts;
// we can keep the most-significant N bits to map the fraction.
// For the purposes of Mandelbrot, it is sufficient to saturate at 4 (|z| >= 2).

// TODO: implement for posits. How?
