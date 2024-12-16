//! Fractal Overdrive as an embeddable widget.

use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlElement, HtmlInputElement};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct Context {
    canvas: HtmlCanvasElement,
    x: HtmlInputElement,
    y: HtmlInputElement,
    window: HtmlInputElement,
    scale: HtmlInputElement,
    iterations: HtmlInputElement,
    resolution: HtmlInputElement,
}

impl Context {
    fn update(&self) {
        log("got re-rendering request");
    }
}
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
    let go: HtmlInputElement = context
        .query_selector("#input-go")?
        .ok_or("no go element in selected region")?
        .dyn_into()?;

    let context = Rc::new(Context {
        canvas,
        x,
        y,
        window,
        scale,
        iterations,
        resolution,
    });

    let trigger = Closure::<dyn Fn()>::new(move || context.update());
    go.add_event_listener_with_callback("submit", trigger.as_ref().unchecked_ref())?;
    // We leak the trigger, per wasm-bindgen advice.
    // This is OK, we will only have one per widget on the page.
    trigger.forget();

    Ok(())
}
