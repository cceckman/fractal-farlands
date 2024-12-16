//! Fractal Overdrive as an embeddable widget.

use std::{num::ParseIntError, rc::Rc};

use ff_core::{CommonParams, NeverCancel, Size};
use num::{bigint::ParseBigIntError, BigInt, BigRational};
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlElement, HtmlFormElement,
    HtmlInputElement, HtmlOptionElement, HtmlSelectElement, ImageData,
};

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

#[derive(Debug)]
struct Context {
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

impl Context {
    /// Extract the common parameters from the
    fn common_params(&self) -> Result<(CommonParams, usize), Error> {
        let [x, y, window, scale]: [Result<num::BigInt, _>; 4] =
            [&self.x, &self.y, &self.window, &self.scale].map(|v| v.value().parse());
        let [x, y, window, scale] = [x?, y?, window?, scale?];
        // Invert the Y axis, to go from "screen dimensions" to "coordinate dimensions".
        let y = -y;

        let half_range = window / 2;
        let range = |v: BigInt| {
            let start = BigRational::new(&v - &half_range, scale.clone());
            let end = BigRational::new(v + &half_range, scale.clone());
            start..end
        };
        let size: usize = self.resolution.value().parse()?;
        let iters: usize = self.iterations.value().parse()?;

        Ok((
            CommonParams {
                size: Size {
                    width: size,
                    height: size,
                },
                x: range(x),
                y: range(y),
                numeric: self.numeric.value(),
            },
            iters,
        ))
    }

    /// Update the "numeric format" options with those available, based on the current fractal.
    fn update_options(&self) {
        let numerics = if self.fractal.value() == "mandelbrot" {
            ff_core::mandelbrot::formats().collect()
        } else if self.fractal.value() == "newton" {
            ff_core::newton::formats().collect()
        } else {
            Vec::new()
        };

        for i in (0..self.numeric.options().length()).rev() {
            let _ = self.numeric.options().remove(i as i32);
        }

        for n in numerics {
            let elem = HtmlOptionElement::new().unwrap();
            elem.set_value(n);
            elem.set_text(n);
            self.numeric.add_with_html_option_element(&elem).unwrap();
        }
    }

    fn update(&self, ev: Option<Event>) {
        if let Some(ev) = ev {
            ev.prevent_default();
        }
        let e = self.render();
        match e {
            Ok(_) => log("rendered"),
            Err(e) => log(&format!("error in rendering: {}", e)),
        }
    }

    fn render(&self) -> Result<(), Error> {
        let (params, iters) = self.common_params()?;
        log(&format!("params: {params:?}"));

        let v = NeverCancel();

        let image = if self.fractal.value() == "mandelbrot" {
            let output = ff_core::mandelbrot::compute(&v, &params, iters).map_err(Error::Render)?;
            render::mandelbrot(output)
        } else if self.fractal.value() == "newton" {
            let output = ff_core::newton::compute(&v, &params, iters).map_err(Error::Render)?;
            render::newton(output)
        } else {
            return Err(Error::InvalidFractal(self.fractal.value()));
        };

        self.canvas.set_height(params.size.height as u32);
        self.canvas.set_width(params.size.width as u32);

        let id = ImageData::new_with_js_u8_clamped_array(&image, params.size.width as u32)
            .map_err(|_| Error::Render("could not create ImageData".to_owned()))?;
        let ctx2d: CanvasRenderingContext2d = self
            .canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        ctx2d
            .put_image_data(&id, 0.0, 0.0)
            .map_err(|e| Error::Render(format!("could not set image data: {:?}", e)))
    }
}

mod render;

/// Insert a Fractal Overdrive widget in the provided context.
#[wasm_bindgen]
pub fn attach(context: HtmlElement) -> Result<(), JsValue> {
    let canvas: HtmlCanvasElement = context
        .query_selector("canvas")?
        .ok_or("no canvas element in selected region")?
        .dyn_into()?;
    let x: HtmlInputElement = context
        .query_selector("#input-x")?
        .ok_or("no X element in selected region")?
        .dyn_into()?;
    let y: HtmlInputElement = context
        .query_selector("#input-y")?
        .ok_or("no Y element in selected region")?
        .dyn_into()?;
    let window: HtmlInputElement = context
        .query_selector("#input-window")?
        .ok_or("no window element in selected region")?
        .dyn_into()?;
    let scale: HtmlInputElement = context
        .query_selector("#input-scale")?
        .ok_or("no scale element in selected region")?
        .dyn_into()?;
    let iterations: HtmlInputElement = context
        .query_selector("#input-iterations")?
        .ok_or("no iterations element in selected region")?
        .dyn_into()?;
    let resolution: HtmlInputElement = context
        .query_selector("#input-resolution")?
        .ok_or("no resolution element in selected region")?
        .dyn_into()?;
    let numeric: HtmlSelectElement = context
        .query_selector("#input-numeric")?
        .ok_or("no numeric element in selected region")?
        .dyn_into()?;
    let fractal: HtmlSelectElement = context
        .query_selector("#input-fractal")?
        .ok_or("no fractal element in selected region")?
        .dyn_into()?;

    let form: HtmlFormElement = context
        .query_selector("form")?
        .ok_or("no formelement in selected region")?
        .dyn_into()?;

    let context = Rc::new(Context {
        canvas,
        x,
        y,
        window,
        scale,
        iterations,
        resolution,
        numeric,
        fractal: fractal.clone(),
    });

    let numeric_trigger = {
        let context = Rc::clone(&context);
        Closure::<dyn Fn(Event)>::new(move |_ev: Event| context.update_options())
    };
    fractal
        .add_event_listener_with_callback("change", numeric_trigger.as_ref().unchecked_ref())
        .unwrap();

    context.update_options();
    context.update(None);

    let trigger = Closure::<dyn Fn(Event)>::new(move |ev: Event| context.update(Some(ev)));
    form.add_event_listener_with_callback("submit", trigger.as_ref().unchecked_ref())?;
    // We leak the triggers, per wasm-bindgen advice.
    // This is OK, we will only have one per widget on the page.
    trigger.forget();
    numeric_trigger.forget();

    Ok(())
}
