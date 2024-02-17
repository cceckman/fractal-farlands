use std::{any::type_name, ops::Range};

/// Implementation of the Mandelbrot fractal,
/// parameterized on a numeric type.
use crate::{numeric::Complex, Size};

mod number;
use num::BigRational;
pub use number::MandelbrotNumber;

/// All-in-one routine for evaluating a portion of the Mandelbrot fractal.
pub fn evaluate<N>(
    x_bounds: &Range<BigRational>,
    y_bounds: &Range<BigRational>,
    size: Size,
    iterations: usize,
) -> Result<Vec<Option<usize>>, String> where N: MandelbrotNumber {
    let mut eval = MandelbrotEval::<N>::new(x_bounds, y_bounds, size)?;
    // TODO: incremental evaluation; check for cancellation.
    eval.advance(iterations);
    Ok(eval.state())
}


/// Type-erased, state-preserving Mandelbrot evaluator.
/// 
/// A Mandelbrot represents a "current state" of the Mandelbrot fractal over a given coordinate window,
/// at a particular number of iterations.
/// It can be advanced without losing that state.
/// 
/// It returns its state as a vector of escapes: at which iteration each coordinate's value escaped past
/// the existing bounds.
pub trait Mandelbrot {
    /// Returns a name for this Mandelbrot evaluator.
    fn name(&self) -> &str;

    /// Returns the (x,y) dimensions of this evaluator.
    fn size(&self) -> Size;

    /// Returns the number of iterations passed so far.
    fn num_iters(&self) -> usize;

    /// Advances this Mandelbrot evaluator by the given number of iterations.
    fn advance(&mut self, num_iters: usize);

    /// Returns the state of this Mandelbrot evaluator:
    /// which cells have escaped, and in what iteration they escaped.
    ///
    /// Cells are presented in row-major order, i.e. [y][x], according to the dimensions in `size`.
    fn state(&self) -> Vec<Option<usize>>;
}

/// A state-preserving Mandelbrot evaluator.
pub struct MandelbrotEval<N> {
    /// For debug / display: the name of this evaluator.
    name: String,

    /// For debug/display: the size of this evaluator.
    size: Size,

    /// Number of iterations completed.
    iterations: usize,

    /// State: coordinate-and-trace pair, at the corresponding number of iterations.
    state: Vec<MandelbrotCell<N>>,
}



impl<N> MandelbrotEval<N>
where
    N: MandelbrotNumber,
{
    /// Construct a new evaluator for the given type.
    ///
    /// The evaluator traces size (x,y) points within the X and Y bounds.
    pub fn new(
        x_bounds: &Range<BigRational>,
        y_bounds: &Range<BigRational>,
        size: Size,
    ) -> Result<Self, String> {
        let x = Self::make_coords(x_bounds, size.x)?;
        let y = Self::make_coords(y_bounds, size.y)?;

        let mut state = Vec::with_capacity(size.x * size.y);

        // Order cells in row-major order, as is typical for graphics.
        for y in y.into_iter() {
            for x in x.iter() {
                state.push(MandelbrotCell::new(Complex {
                    re: x.clone(),
                    im: y.clone(),
                }));
            }
        }

        let name = format!("Mandelbrot({})", type_name::<N>());
        Ok(Self {
            name,
            size,
            iterations: 0,
            state,
        })
    }

    /// Produce a range of `size` coordinates between the given bounds.
    fn make_coords(bounds: &Range<BigRational>, size: usize) -> Result<Vec<N>, String> {
        // We may not have enough integer precision in our type to represent `size`;
        // we only need three bits of integer to faithfully compute z^2 and compare to 4.
        // So: approximate the range, then convert each.
        let big_size = BigRational::new(size.into(), 1.into());
        let step = ((bounds.end.clone() - bounds.start.clone()) / big_size).reduced();
        let mut coord = bounds.start.clone();
        let mut results = Vec::with_capacity(size);
        for _ in 0..size {
            let value = N::from_bigrational(&coord).ok_or_else(|| {
                format!(
                    "cannot approximate coordinate {} as a {}",
                    &coord,
                    std::any::type_name::<N>()
                )
            })?;
            results.push(value);
            coord += step.clone();
        }
        Ok(results)
    }
}

impl<N> Mandelbrot for MandelbrotEval<N>
where
    N: MandelbrotNumber,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn size(&self) -> Size {
        self.size
    }

    fn num_iters(&self) -> usize {
        self.iterations
    }

    fn advance(&mut self, num_iters: usize) {
        for cell in self.state.iter_mut() {
            cell.update(self.iterations, num_iters)
        }
        self.iterations += num_iters
    }

    fn state(&self) -> Vec<Option<usize>> {
        self.state
            .iter()
            .map(|cell| match cell.state {
                TraceState::Active(_) => None,
                TraceState::Escaped(iters) => Some(iters),
            })
            .collect()
    }
}

/// Cell of Mandelbrot evaluation.
struct MandelbrotCell<N> {
    /// Coordinate on the complex plane.
    coordinate: Complex<N>,

    /// Current value of the trace.
    state: TraceState<N>,
}

/// State of a point in Mandelbrot evaluation:
/// either still within the |z| < 2 circle, or escaped after N iterations.
enum TraceState<N> {
    Active(Complex<N>),
    Escaped(usize),
}

impl<N> MandelbrotCell<N>
where
    N: MandelbrotNumber,
{
    /// Construct a new MandelbrotCell at the given coordinate.
    fn new(coordinate: Complex<N>) -> Self {
        let state = Complex {
            re: N::zero(),
            im: N::zero(),
        };
        Self {
            coordinate,
            state: TraceState::Active(state),
        }
    }

    /// Step through the trace of this coordinate for `iters_more` additional iterations.
    /// Update to "escaped"
    fn update(&mut self, iters_past: usize, iters_more: usize) {
        let z = match &mut self.state {
            TraceState::Escaped(_) => return,
            TraceState::Active(v) => v,
        };
        let four: N = N::four();

        for i in 0..iters_more {
            *z = z.clone() * z.clone() + self.coordinate.clone();
            let z_magnitude_squared = z.re.clone() * z.re.clone() + z.im.clone() * z.im.clone();

            // The Mandelbrot "escape condition" is that the Cartesian distance from the zero point
            // of the complex plane (0 + 0i) is at least two.
            // Normally, that distance is sqrt(x^2+y^2) - but we can skip the square-root and avoid
            // a trait requirement by comparing d^2 to 2^2 instead:
            if z_magnitude_squared >= four {
                self.state = TraceState::Escaped(iters_past + i);
                return;
            }
        }
    }
}
