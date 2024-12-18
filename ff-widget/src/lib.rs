//! Fractal Overdrive as an embeddable widget.

use std::num::ParseIntError;

use ff_core::{CommonParams, NeverCancel, Size};
use num::{bigint::ParseBigIntError, BigInt, BigRational};
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("invalid numeric input: {0}")]
    InvalidBigInt(#[from] ParseBigIntError),

    #[error("invalid numeric input: {0}")]
    InvalidInt(#[from] ParseIntError),

    #[error("failed to render: {0}")]
    Render(String),

    #[error("invalid fractal: {0}")]
    InvalidFractal(String),
}

/// A rendering request from the JS side.
#[wasm_bindgen]
#[derive(Default, Debug)]
pub struct Request {
    /// Name of the widget.
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,

    #[wasm_bindgen(getter_with_clone)]
    pub x: String,
    #[wasm_bindgen(getter_with_clone)]
    pub y: String,
    #[wasm_bindgen(getter_with_clone)]
    pub half_window: String,
    #[wasm_bindgen(getter_with_clone)]
    pub scale: String,
    #[wasm_bindgen(getter_with_clone)]
    pub iterations: String,
    #[wasm_bindgen(getter_with_clone)]
    pub resolution: String,
    #[wasm_bindgen(getter_with_clone)]
    pub numeric: String,
    #[wasm_bindgen(getter_with_clone)]
    pub fractal: String,
}

/// Attempt to reduce the fraction ((x, y) +- half_window) / scale
/// by a common factor.
/// Returns the greatest common divisor between all numbers -- which may be 1.
#[wasm_bindgen]
pub fn reduce(x: String, y: String, half_window: String, scale: String) -> String {
    reduce_rs(x, y, half_window, scale).unwrap_or_else(|e| {
        log(&format!("error reducing fraction: {e}"));
        "1".to_owned()
    })
}

fn reduce_rs(x: String, y: String, half_window: String, scale: String) -> Result<String, Error> {
    let [x, y, half_window, scale]: [num::BigInt; 4] =
        [x.parse()?, y.parse()?, half_window.parse()?, scale.parse()?];
    use num::Integer;
    let gcd = x.gcd(&y).gcd(&half_window).gcd(&scale);
    Ok(gcd.to_string())
}

#[wasm_bindgen]
impl Request {
    /// Create a new rendering request.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    /// Inner Rust routine to render,
    /// but generates a non-JS-friendly error type.
    fn render_rs(&self) -> Result<ImageData, Error> {
        let (params, iters) = self.common_params()?;

        let v = NeverCancel();

        let image = if self.fractal == "mandelbrot" {
            let output = ff_core::mandelbrot::compute(&v, &params, iters).map_err(Error::Render)?;
            render::mandelbrot(output)
        } else if self.fractal == "newton" {
            let output = ff_core::newton::compute(&v, &params, iters).map_err(Error::Render)?;
            render::newton(output)
        } else {
            return Err(Error::InvalidFractal(self.fractal.clone()));
        };

        //self.canvas.set_height(params.size.height as u32);
        //self.canvas.set_width(params.size.width as u32);

        ImageData::new_with_js_u8_clamped_array(&image, params.size.width as u32)
            .map_err(|_| Error::Render("could not create ImageData".to_owned()))
    }

    /// Render the fractal given the provided parameters.
    #[wasm_bindgen]
    pub fn render(&self) -> Result<ImageData, String> {
        self.render_rs().map_err(|e| e.to_string())
    }
}

/// Get a list of the numeric types that can be used with this fractal.
#[wasm_bindgen]
pub fn numeric_options(fractal: &str) -> Vec<String> {
    if fractal == "mandelbrot" {
        ff_core::mandelbrot::formats()
            .map(|v| v.to_owned())
            .collect()
    } else if fractal == "newton" {
        ff_core::newton::formats().map(|v| v.to_owned()).collect()
    } else {
        Vec::new()
    }
}

impl Request {
    /// Extract the common parameters from the
    fn common_params(&self) -> Result<(CommonParams, usize), Error> {
        let [x, y, half_window, scale]: [Result<num::BigInt, _>; 4] =
            [&self.x, &self.y, &self.half_window, &self.scale].map(|v| v.parse());
        let [x, y, half_window, scale] = [x?, y?, half_window?, scale?];
        // Invert the Y axis, to go from "screen dimensions" to "coordinate dimensions".
        let y = -y;

        let range = |v: BigInt| {
            let start = BigRational::new(&v - &half_window, scale.clone());
            let end = BigRational::new(v + &half_window, scale.clone());
            start..end
        };
        let size: usize = self.resolution.parse()?;
        let iters: usize = self.iterations.parse()?;

        Ok((
            CommonParams {
                size: Size {
                    width: size,
                    height: size,
                },
                x: range(x),
                y: range(y),
                numeric: self.numeric.clone(),
            },
            iters,
        ))
    }
}

mod render;
