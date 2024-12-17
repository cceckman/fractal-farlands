//! Fractal Overdrive as an embeddable widget.

use std::num::ParseIntError;

use ff_core::{CommonParams, NeverCancel, Size};
use num::{bigint::ParseBigIntError, BigInt, BigRational};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlCanvasElement, HtmlInputElement, HtmlSelectElement, ImageData};

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
    pub window: String,
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

#[wasm_bindgen]
impl Request {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
struct Context {
    name: String,
    canvas: HtmlCanvasElement,
    x: HtmlInputElement,
    y: HtmlInputElement,
    window: HtmlInputElement,
    scale: HtmlInputElement,
    iterations: HtmlInputElement,
    resolution: HtmlInputElement,
    numeric: HtmlSelectElement,
    fractal: HtmlSelectElement,
}

/// Get a list of strings-- numeric types that can be used with this fractal.
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
        let [x, y, window, scale]: [Result<num::BigInt, _>; 4] =
            [&self.x, &self.y, &self.window, &self.scale].map(|v| v.parse());
        let [x, y, window, scale] = [x?, y?, window?, scale?];
        // Invert the Y axis, to go from "screen dimensions" to "coordinate dimensions".
        let y = -y;

        let half_range = window / 2;
        let range = |v: BigInt| {
            let start = BigRational::new(&v - &half_range, scale.clone());
            let end = BigRational::new(v + &half_range, scale.clone());
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

    fn update(&self, ev: Option<Event>) {
        assert!(!self.name.is_empty());
        // log(&format!("updating {}", &self.name));
        if let Some(ev) = ev {
            ev.prevent_default();
        }
        let e = self.render();
        match e {
            Ok(_) =>
            /*log(&format!("rendered {}", &self.name))*/
            {
                ()
            }
            Err(e) => log(&format!(
                "error in rendering: {}, {}",
                /*&self.name*/ "widget", e
            )),
        }
    }

    fn render(&self) -> Result<(), Error> {
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

        let id = ImageData::new_with_js_u8_clamped_array(&image, params.size.width as u32)
            .map_err(|_| Error::Render("could not create ImageData".to_owned()))?;
        //let ctx2d: CanvasRenderingContext2d = self
        //    .canvas
        //    .get_context("2d")
        //    .unwrap()
        //    .unwrap()
        //    .dyn_into()
        //    .unwrap();
        //ctx2d
        //    .put_image_data(&id, 0.0, 0.0)
        //    .map_err(|e| Error::Render(format!("could not set image data: {:?}", e)))
        Ok(())
    }
}

mod render;
