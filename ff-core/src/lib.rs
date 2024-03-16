//! Library code for Fractal Farlands.

use std::ops::Range;

use num::BigRational;

pub mod mandelbrot;
pub mod masked_float;
mod numeric;

pub use numeric::FromRational;


/// Rendering-request parameters, common across renderables.
#[derive(Debug,Clone)]
pub struct CommonParams {
    /// Rendered size, in pixels
    pub size: Size,

    /// X bounds, in rational coordinates
    pub x: Range<BigRational>,
    /// Y bounds, in rational coordinates
    pub y: Range<BigRational>,

    /// Numeric type to use for the computations.
    /// This is assumed to be "mappable" by the rendering engine.
    pub numeric: String,
}

/// Fractal-specific rendering parameters.
#[non_exhaustive]
#[derive(Debug,Clone)]
pub enum FractalParams {
    Mandelbrot{
        iters: usize,
    },
}

impl FractalParams {
    pub fn name(&self) -> &'static str {
        match self {
            FractalParams::Mandelbrot{..} => "mandelbrot",
        }
    }
}

/// Request for rendering a fractal.
#[derive(Debug,Clone)]
pub struct RenderRequest{
    pub common: CommonParams,
    pub fractal: FractalParams,
}

/// A pair of integer (x, y) dimensions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

pub mod image;

/// Escape term: on what iteration the escape occurred, and with what value.
#[derive(Copy,Clone,Debug)]
pub struct Escape {
    pub count: usize,
    pub z_magnitude_squared: f64,
}

/// Shorthand for "the escapes for this region"
pub type EscapeVector = Vec<Option<Escape>>;


