use std::ops::{Add, Mul, Sub};

mod mandelbrot;

/// A type which adds, multiplies, and subtracts to itself.
/// This is a marker trait, with a default implementation;
/// it just makes this listing more convenient.
trait AddMulSub:
    Sized + Mul<Self, Output = Self> + Add<Self, Output = Self> + Sub<Self, Output = Self>
{
}

impl<N> AddMulSub for N where
    N: Sized + Mul<Self, Output = Self> + Add<Self, Output = Self> + Sub<Self, Output = Self>
{
}

/// Complex number implementation.
/// A little more granular than num_traits, because we're only interested in certain ops.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Complex<N> {
    pub re: N,
    pub im: N,
}

impl<N> Mul<Complex<N>> for Complex<N>
where
    N: Clone + AddMulSub,
{
    type Output = Complex<N>;

    fn mul(self, rhs: Complex<N>) -> Self {
        // (a + ib) * (c + id)
        // = ac + aid + (ibc + i^2 bd)      (FOIL)
        // = (ac - bd) + i(ad + bc)         (turning i^2 into -1, combining real/imaginary terms)
        let (a, b) = (self.re, self.im);
        let (c, d) = (rhs.re, rhs.im);
        let re = a.clone() * c.clone() - b.clone() * d.clone();
        let im = a * d + b * c;
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

/// Trait for conversion from a BigRational.
trait ApproximateFromBigRational {
    /// Approximates the given BigRational as closely as this format can.
    fn approximate(value: &num::BigRational) -> Self;
}
