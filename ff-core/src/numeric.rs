use std::ops::{Add, Div, Mul, Sub};

use num::{BigRational, ToPrimitive};

use crate::{mandelbrot::MandelbrotNumber, masked_float::MaskedFloat};

/// A numeric type that can be converted from a BigRational.
///
/// This is provided as a distinct trait because we can't expect `From<BigRational>`
/// on foreign types.
pub trait FromRational {
    fn from_bigrational(r: &BigRational) -> Result<Self, String>
    where
        Self: Sized;
}

impl FromRational for f32 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String>
    where
        Self: Sized,
    {
        r.to_f32().ok_or(format!("failed conversion from {}", r))
    }
}

impl FromRational for f64 {
    fn from_bigrational(r: &BigRational) -> Result<Self, String>
    where
        Self: Sized,
    {
        r.to_f64().ok_or(format!("failed conversion from {}", r))
    }
}

impl<const E: usize, const F: usize> FromRational for MaskedFloat<E, F> {
    fn from_bigrational(value: &BigRational) -> Result<Self, String> {
        let f: f64 = f64::from_bigrational(value)?;
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
{
    /// Squares the given number.
    /// Per https://github.com/cceckman/fractal-farlands/issues/9, this takes fewer operations than
    /// a generic multiply.
    pub fn square(self) -> Self {
        // (a+bi)^2 = (a^2-b^2) + 2abi
        let re = self.re.clone() * self.re.clone() - self.im.clone() * self.im.clone();
        let im = <N as MandelbrotNumber>::two() * (self.re * self.im);
        Self { re, im }
    }

    /// Reports if two complex numbers are near each other.
    ///
    /// Near is defined as:
    ///   If the distance between the two is less than the magnitude of nb divided by
    ///   'threshold', then the numbers are near.
    ///
    /// It's defined this way to avoid having to reason more carefully about epsilon for the
    /// various formats. 'nb' should be similar to self and rhs, ideally the larger of the two.
    ///
    /// Based on this blog post, where the method is called "not bad" and says it "mostly" works.
    /// https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/
    pub fn near(&self, rhs: Complex<N>, nb: Complex<N>, threshold: N) -> bool {
        let nearby = (nb.re.clone() * nb.re.clone() + nb.im.clone() * nb.im.clone())/threshold;
        let dre = self.re.clone() - rhs.re.clone();
        let dim = self.im.clone() - rhs.im.clone();
        let distance = dre.clone() * dre + dim.clone() * dim;
        return distance < nearby;
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

impl<N> Div<Complex<N>> for Complex<N>
where
    N: Clone + Add<N, Output = N> + Sub<N, Output = N> + Mul<N, Output = N> + Div<N, Output = N>,
{
    type Output = Complex<N>;

    fn div(self, rhs: Complex<N>) -> Self {
        // https://mathworld.wolfram.com/ComplexDivision.html
        let (a, b) = (self.re, self.im);
        let (c, d) = (rhs.re, rhs.im);
        let re: N = (a.clone() * c.clone() + b.clone() * d.clone())
            / (c.clone() * c.clone() + d.clone() * d.clone());
        let im: N = (b.clone() * c.clone() - a.clone() * d.clone())
            / (c.clone() * c.clone() + d.clone() * d.clone());
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

impl<N> Sub<Complex<N>> for Complex<N>
where
    N: Sub<N, Output = N>,
{
    type Output = Complex<N>;

    fn sub(self, rhs: Complex<N>) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl<N> Add<&Complex<N>> for Complex<N>
where
    for<'a> N: Add<&'a N, Output = N>,
{
    type Output = Complex<N>;

    fn add(self, rhs: &Complex<N>) -> Self::Output {
        let re: N = self.re + &rhs.re;
        let im: N = self.im + &rhs.im;
        Complex { re, im }
    }
}
