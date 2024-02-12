//! Library code for Fractal Farlands.

pub mod mandelbrot;
mod numeric;

/// A pair of integer (x, y) dimensions.
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Size {
    pub x: usize,
    pub y: usize,
}


pub mod image;
