use std::ops::{Add, Mul, Sub};

/// Complex number implementation.
/// A little more granular than num_traits, because we're only interested in certain ops.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Complex<N> {
    pub re: N,
    pub im: N,
}

impl<'a, 'b, N> Mul<&'b Complex<N>> for &'a Complex<N>
where
    &'a N: Add<&'b N, Output = N> + Sub<&'b N, Output = N> + Mul<&'b N, Output = N>,
{
    type Output = Complex<N>;

    fn mul(self, rhs: &Complex<N>) -> Complex<N> {
        // (a + ib) * (c + id)
        // = ac + aid + (ibc + i^2 bd)      (FOIL)
        // = (ac - bd) + i(ad + bc)         (turning i^2 into -1, combining real/imaginary terms)
        let (a, b) = (&self.re, &self.im);
        let (c, d) = (&rhs.re, &rhs.im);
        let re: N = &(a * c) - &(b * d);
        let im: N = &(a * d) + &(b * c);
        Complex { re, im }
    }
}

impl<'a, 'b, N> Add<&'b Complex<N>> for &'a Complex<N>
where
    &'a N: Add<&'b N, Output = N>,
{
    type Output = Complex<N>;

    fn add(self, rhs: &Complex<N>) -> Complex<N> {
        Complex {
            re: &self.re + &rhs.re,
            im: &self.im + &rhs.im,
        }
    }
}

#[cfg(test)]
mod tests {
    use num::BigRational;

    use super::Complex;

    #[test]
    fn complex_f32() {
        let x = Complex{re: 1f32, im: 2f32};
        let y = x.clone();
        let got  = &x * &y;
        let want = Complex{re: -3f32, im: 4f32};
        assert_eq!(got, want);
    }

    #[test]
    fn complex_f64() {
        let x = Complex{re: 1f64, im: 2f64};
        let y = x.clone();
        let got  = &x * &y;
        let want = Complex{re: -3f64, im: 4f64};
        assert_eq!(got, want);
    }

    #[test]
    fn complex_big_rational() {
        let (one, two, neg_three, four) = (
            BigRational::new(1.into(), 1.into()),
            BigRational::new(2.into(), 1.into()),
            BigRational::new((-3).into(), 1.into()),
            BigRational::new(4.into(), 1.into()),
        );


        let x = Complex{re: one, im: two};
        let y = x.clone();
        let got  = &x * &y;
        let want = Complex{re: neg_three, im: four};
        assert_eq!(got, want);
    }

}