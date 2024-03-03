use std::ops::{Add, Mul, Sub};

use num::{BigRational, ToPrimitive};

use crate::{mandelbrot::MandelbrotNumber, masked_float::MaskedFloat};

/// A numeric type that can be converted from a BigRational.
///
/// This is provided as a distinct trait because we can't expect `From<BigRational>`
/// on foreign types.
pub trait FromRational {
    fn from_bigrational(r: &BigRational) -> Result<Self, String> where Self: Sized;
}

impl FromRational for f32 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String> where Self: Sized {
        r.to_f32().ok_or(format!("failed conversion from {}", r))
    }
}

impl FromRational for f64 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String> where Self: Sized {
        r.to_f64().ok_or(format!("failed conversion from {}", r))
    }
}

impl<const E: usize, const F: usize> FromRational for MaskedFloat<E, F> {
    fn from_bigrational(value: &BigRational) -> Result<Self, String> {
        let f : f64 = f64::from_bigrational(value)?;
        Ok(MaskedFloat::<E, F>::new(f))
    }
}

impl FromRational for BigRational {
    fn from_bigrational(value: &BigRational) -> Result<Self, String> {
        Ok(value.clone())
    }
}

/// Complex number implementation.
/// A little more granular than num_traits, because we're only interested in certain ops.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Complex<N> {
    pub re: N,
    pub im: N,
}

impl<N> Complex<N>
where
    N: MandelbrotNumber,
    for<'a> &'a N: Mul<&'a N, Output=N>,
{
    /// Squares the given number.
    /// Per https://github.com/cceckman/fractal-farlands/issues/9, this takes fewer operations than
    /// a generic multiply.
    pub fn square(&self) -> Self {
        // (a+bi)^2 = (a^2-b^2) + 2abi
        let re = &self.re * &self.re - &self.im * &self.im;
        let im = <N as MandelbrotNumber>::two() * (&self.re * &self.im);
        Self {re, im}
    }
}

impl<N> Mul<Complex<N>> for Complex<N>
where
    N: Clone + Add<N, Output = N> + Sub<N, Output = N> + Mul<N, Output = N>,
{
    type Output = Complex<N>;

    fn mul(self, rhs: Complex<N>) -> Self {
        // (a + ib) * (c + id)
        // = ac + aid + (ibc + i^2 bd)      (FOIL)
        // = (ac - bd) + i(ad + bc)         (turning i^2 into -1, combining real/imaginary terms)
        let (a, b) = (self.re, self.im);
        let (c, d) = (rhs.re, rhs.im);
        let re: N = (a.clone() * c.clone()) - (b.clone() * d.clone());
        let im: N = a * d + b * c;
        Self { re, im }
    }
}

impl<N> Add<Complex<N>> for Complex<N>
where
    N: Add<N, Output = N>,
{
    type Output = Complex<N>;

    fn add(self, rhs: Complex<N>) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
