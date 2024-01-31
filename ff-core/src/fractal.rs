//! Evaluation utilities of fractals.

use num::BigRational;
use std::ops::Range;

/// Evaluate the provided fractals in the given range.
pub fn evaluate_all(
    iteration_limit: u32,
    x: (&BigRational, &BigRational),
    y: (&BigRational, &BigRational),
    size: (usize, usize),
    evaluators: &[&dyn FractalEval],
) -> Vec<EvalResult> {
    /// Prepare buffers:
    let mut result = evaluators.iter().map(|eval| {
        let name = eval.name().to_owned();
        let mut output : Vec<Option<u32>> = Vec::new();
        output.resize(size.0 * size.1, None);
        EvalResult {
            name,
            size: size.clone(),
            output,
        }
    });
    // TODO: Eh?

    result
}

/// Result of a fractal evaluation.
struct EvalResult {
    name: String,
    size: (usize, usize),
    output: Vec<Option<u32>>
}



/// Evaluator for a fractal.
///
/// This trait allows evaluation and comparison of a fractal, dispatched and annotated dynamically.
pub trait FractalEval {
    /// Provides a descriptive name for this evaluator, e.g. `float32` or `posit16`.
    fn name(&self) -> &str;

    /// Evaluate the fractal in a particular range.
    ///
    /// The Y-coordinate is fixed for the entire call;
    /// all points are to be evaluated at the same Y coordinate.
    /// The call provides a list of X-coordinates to evaluate at.
    ///
    /// The output buffer must be the same length as the list of X-coordinates.
    ///
    /// Both coordinates are provided as BigRational, and are therefore representing an exact (pixel)
    /// coordinate; the implementation of N may lose precision when converting these to their internal
    /// representation.
    fn eval_range(&self,
        iteration_limit: u32,
        y: &BigRational,
        input_x: &[BigRational],
        output: &mut [Option<u32>],
    );
}




#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
	}
}
