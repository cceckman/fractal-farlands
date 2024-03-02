//! Library code for Fractal Farlands.

use std::ops::Range;

use num::BigRational;

pub mod mandelbrot;
pub mod masked_float;
mod numeric;


/// Rendering-request parameters, common across renderables.
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
