use fixed::types::{I11F5, I20F12};
use rayon::prelude::*;
use std::{ops::{Mul, Range}, panic::AssertUnwindSafe};

// Implementation of Newton's fractal for z^3-1
// TODO:
//   Parameterize to other functions
use crate::{masked_float::MaskedFloat, numeric::Complex, CancelContext, CommonParams};

pub use crate::number::FractalNumber;
use crate::{Zero, ZeroVector};
use num::BigRational;

/// Parameters for the Newton fractal.
pub struct NewtonParams {
    pub iters: usize,
    // A Newton polynomial can be factored as:
    // - A set of zero terms, each of which represents a term (z-c)
    // - A multiplier across all those terms
    // That is: k(z-c1)(z-c2)(z-c3)...
    pub zeros: Vec<Complex<BigRational>>,
    pub coefficient: Complex<BigRational>,
}

/// A single polynomial term: k z^N
#[derive(PartialEq,Eq,Debug,Ord,Clone)]
struct PolynomialTerm {
    power: isize,
    coefficient: Complex<BigRational>,
}

impl Into<PolynomialTerm> for Complex<BigRational> {
    fn into(self) -> PolynomialTerm {
        PolynomialTerm{
            coefficient: self,
            power: 0,
        }
    }
}

impl std::ops::Mul<&PolynomialTerm> for &PolynomialTerm {
    type Output = PolynomialTerm;

    fn mul(self, rhs: &PolynomialTerm) -> Self::Output {
        // a z^N * b z^M = (ab) z^(N+M)
        PolynomialTerm{
            coefficient: &rhs.coefficient * &self.coefficient,
            power: rhs.power + self.power,
        }
    }
}

impl PartialOrd for PolynomialTerm {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.power.partial_cmp(&other.power) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.coefficient.partial_cmp(&other.coefficient)
    }
}

#[derive(Clone)]
struct Polynomial(Vec<PolynomialTerm>);

impl Polynomial {
    /// Reduce duplicate terms in the polynomial.
    fn reduce(self) -> Polynomial{
        let mut terms = self.0;
        terms.sort();
        let mut result : Vec<PolynomialTerm> = Default::default();
        for term in terms.into_iter() {
            // Merge terms with the same power.
            // We're iterating in order of power, so we only have to check the last item.
            if let Some(last) = result.last_mut() {
                if term.power == last.power {
                    last.coefficient = term.coefficient + &last.coefficient;
                    continue
                }
            }
            result.push(term)
        }
        Polynomial(result)
    }

    /// Take the first derivative of this polynomial.
    /// Assumes the polynomial is already reduced.
    fn derive(&self) -> Polynomial {
        let new_terms : Vec<PolynomialTerm> = self.0.iter().filter_map(|term| {
            if term.power == 0 {
                None
            } else {
                Some({
                    let re = &term.coefficient.re * BigRational::new(term.power.into(), 1.into());
                    let im = &term.coefficient.im * BigRational::new(term.power.into(), 1.into());
                    let coefficient = Complex { re, im};
                    let power = term.power - 1;
                    PolynomialTerm{
                        coefficient, power
                    }
                })
            }
        }).collect();
        Polynomial(new_terms)
    }

    /// Returns a function that can evaluate this polynomial over complex numbers.
    fn evaluator<N>(&self) -> Result<Box<dyn Fn(&Complex<N>) -> Complex<N> + Send + Sync>, String>
    where N: FractalNumber + Send + Sync + 'static
    {
        // Convert the terms to the FractalNumber
        let terms : Result<Vec<_>, String> = self.0.iter().map(|x| -> Result<(Complex<N>, isize), String> {
            let re = N::from_bigrational(&x.coefficient.re)?;
            let im = N::from_bigrational(&x.coefficient.im)?;
            Ok((Complex{re, im}, x.power))
        }).collect();
        let terms : Vec<(Complex<N>, isize)> = terms?;
        Ok(Box::new(move |z: &Complex<N>| {
            terms.iter().map(|t| {
                let (coefficient, power) = t;
                // Raise to the nth power:
                let mut accumulator = Complex{re: N::from_i32(1), im: N::from_i32(0)};
                for i in 0..(*power) {
                    accumulator = accumulator * z.clone();
                }
                accumulator * coefficient.clone()
            }).reduce(|a, b| a + b).unwrap_or_else(|| {
                Complex{ re: N::from_i32(0), im: N::from_i32(0)}
            })
        }))
    }
}

impl Mul<&Polynomial> for Polynomial {
    type Output = Polynomial;

    fn mul(self, rhs: &Polynomial) -> Self::Output {
        // N^2 then reduce. Eugh.
        let result = self.0.into_iter().flat_map(|term| {
            rhs.0.iter().map(move |other| &term * other)
        }).collect();
        Polynomial(result).reduce()
    }
}

impl Mul<&Complex<BigRational>> for Polynomial {
    type Output = Polynomial;

    fn mul(self, rhs: &Complex<BigRational>) -> Self::Output {
        let result = self.0.into_iter().map(|term| {
            PolynomialTerm{
                coefficient: term.coefficient * rhs,
                power: term.power,
            }
        }).collect();
        Polynomial(result).reduce()
    }
}

struct NewtonEvaluator<N> {
    zeros: Vec<Complex<N>>,

    p: Box<dyn Fn(&Complex<N>) -> Complex<N>>,
    p_prime: Box<dyn Fn(&Complex<N>) -> Complex<N>>,

    iters: usize,
}

impl<N> TryFrom<NewtonParams> for NewtonEvaluator<N> where N: FractalNumber + Send + Sync + 'static {
    type Error = String;

    fn try_from(params: NewtonParams) -> Result<Self, Self::Error> {
        let zeros : Result<Vec<_>, String>= params.zeros.iter().map(|v| {
            let re = N::from_bigrational(&v.re)?;
            let im = N::from_bigrational(&v.im)?;
            Ok(Complex{re, im})
        }).collect();
        let zeros = zeros?;
        let one = Complex { re: BigRational::from_integer(1.into()), im: BigRational::from_integer(1.into())};

        let terms = params.zeros.into_iter().map(|zero| {
            // Each of the zeros represents a polynomial, z^1 - c.
            // Express as a Polynomial:
            Polynomial(vec![
                PolynomialTerm{power: 1, coefficient: one.clone()},
                PolynomialTerm{power: 0, coefficient: Complex{
                    re: -zero.re,
                    im: -zero.im,
                }},
            ])
        }).reduce(|a, b| a * &b).ok_or_else(|| "no zeros given")?;
        // Distribute the initial coefficient across all terms
        let p_exact = terms * &params.coefficient;
        let p_prime_exact = p_exact.derive();
        let p = p_exact.evaluator()?;
        let p_prime = p_prime_exact.evaluator()?;

        Ok(NewtonEvaluator{
            zeros, p, p_prime, iters: params.iters
        })
    }
}


/// Function pointer for evaluating zeros
type EscapeFn = fn(&dyn CancelContext, &CommonParams, usize) -> Result<ZeroVector, String>;

const FUNCTIONS: &[(&'static str, EscapeFn)] = &[
    ("f32", evaluate_parallel_numeric::<f32>),
    ("f64", evaluate_parallel_numeric::<f64>),
    ("P32", evaluate_parallel_numeric::<softposit::P32>),
    ("P16", evaluate_parallel_numeric::<softposit::P16>),
    // P8 and MaskedFloat<3,50> don't produce interesting images, mostly fail to converge.
    //("P8", evaluate_parallel_numeric::<softposit::P8>),
    //("MaskedFloat<3,50>", evaluate_parallel_numeric::<MaskedFloat<3, 50>>),
    (
        "MaskedFloat<4,50>",
        evaluate_parallel_numeric::<MaskedFloat<4, 50>>,
    ),
    ("I20F12", evaluate_parallel_numeric::<I20F12>),
    ("I11F5", evaluate_parallel_numeric::<I11F5>),
];

/// List the numeric formats that are valid for rendering.
pub fn formats() -> impl Iterator<Item = &'static str> {
    FUNCTIONS.iter().map(|(name, _)| *name)
}

pub fn compute(ctx: &dyn CancelContext, params: &CommonParams, iterations: usize) -> Result<ZeroVector, String> {
    // Default to z^3 - 1 = (z-1)(z-i)(z+i)
    // ... which has irrational roots. Crap.
    let newton_params = NewtonParams{
        iters: iterations,
        zeros: vec![Complex{ }
        ]
    };



    let fmt = params.numeric.as_str();
    // Linear scan, we don't have that many options:
    for (candidate, computer) in FUNCTIONS.iter() {
        if *candidate == fmt {
            return computer(ctx, params, iterations);
        }
    }

    Err(format!("unknown numeric format {}", fmt))
}

fn evaluate_parallel_numeric<N>(
    ctx: &dyn CancelContext,
    params: &CommonParams,
    iterations: usize,
) -> Result<ZeroVector, String>
where
    N: FractalNumber + Send + Sync,
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
        .par_bridge()
        .into_par_iter()
        .for_each(|(y, row_out)| {
            if ctx.is_canceled() {
                return
            }

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                xs.iter().zip(row_out).for_each(|(x, out)| {
                    *out = find_zero(x, &y, iterations);
                })
            }));
            if result.is_err() {
                tracing::error!("caught panic during mandelbrot evaluation");
            }
        });

    let mut zero_index: Vec<Complex<N>> = Vec::new();

    if ctx.is_canceled() {
        return Err("canceled".to_string())
    }

    Ok(zeros
        .into_iter()
        .map(|x| match x {
            None => None,
            Some((z, iters)) => match zero_index
                .iter()
                .position(|x| (*x).near(z.clone(), z.clone(), N::from_i32(512)))
            {
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
    N: FractalNumber,
{
    let mut z: Complex<N> = Complex {
        re: x.clone(),
        im: y.clone(),
    };

    let zero: Complex<N> = Complex {
        re: N::from_i32(0),
        im: N::from_i32(0),
    };

    let one: Complex<N> = Complex {
        re: N::from_i32(1),
        im: N::from_i32(0),
    };

    let three: Complex<N> = Complex {
        re: N::from_i32(3),
        im: N::from_i32(0),
    };

    for i in 0..limit {
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
        let fpz = three.clone() * z.clone() * z.clone();
        if fpz.re.clone() * fpz.re.clone() + fpz.im.clone() * fpz.im.clone() == N::from_i32(0) {
            return None;
        }
        let del = fz.clone() / fpz;
        if fz.near(zero.clone(), z.clone(), N::from_i32(1024)) {
            return Some((z, i));
        }
        z = z - del;
    }
    //println!("Fail: Z[{}]: re: {:?} im: {:?}", limit, z.re, z.im);
    return None;
}
