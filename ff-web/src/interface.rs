use crate::WindowParams;
/// User-interface rendering for Fractal Farlands.
use axum::{extract::OriginalUri, extract::Query};

use maud::{html, Markup, DOCTYPE};

/// Render the user interface.
pub async fn interface(uri: OriginalUri, Query(query): Query<WindowParams>) -> Markup {
    html! {
        (DOCTYPE)
        title { "Fractal Farlands" }
        link rel="stylesheet" href="static/style.css";
        body {
            (interface_body(uri.query().unwrap_or(""), &query))
        }
    }
}

fn interface_body(query_str: &str, query: &WindowParams) -> Markup {
    html! {
        form action="/" autocomplete="off" class="parameters" {
            h2 { "Target area" }
            p {
                label { "Center X (numerator):" }
                input name="x" type="number" value=(query.x);
                " "

                label { "Center Y (numerator):" }
                input name="y" type="number" value=(query.y);
                " "

                label { "Window size (numerator)" }
                input name="window" type="number" value=(query.window);
                " "

                label { "Scale (denominator):" }
                input name="scale" type="number" value=(query.scale);
                " "
            }
            h2 { "Rendering settings" }
            p {
                label { "Fractal:" }
                select {
                    option value="mandelbrot" selected=(if query.fractal == "mandelbrot" { true } else { false }){ "Mandelbrot " }
                }
                " "

                label { "Resolution (pixels):" }
                input name="res" type="number" value=(query.res);
                " "

                label { "Max iterations:" }
                input name="iters" type="number" value=(query.iters);
                " "
                br;
            }

            input text="Go" type="submit";
        }

        div {
            h2 { "Rendering results "}
            p { (format!("Parameters: {:?}", query)) }

            (render(query_str, &query.fractal, "f32", query.res))
            (render(query_str, &query.fractal, "f64", query.res))
        }

    }
}

fn render(query_str: &str, fractal: &str, numeric: &str, size: usize) -> Markup {
    html! {
        div class="render-pane" {
            h3 { (numeric) }
            img src=(format!("/render/{}/{}?{}", fractal, numeric, query_str)) width=(size) height=(size) class="renderpane";
        }
    }
}
