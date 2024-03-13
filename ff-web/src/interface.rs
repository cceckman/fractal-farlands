use crate::WindowParams;
/// User-interface rendering for Fractal Farlands.
use axum::{extract::OriginalUri, extract::Query};

use maud::{html, Markup, DOCTYPE};
use num::Integer;

/// Render the user interface.
pub async fn interface(uri: OriginalUri, Query(query): Query<WindowParams>) -> Markup {
    // Simplify WindowParams where we can- before outputting to the user.
    // This may mean our query parameters don't match; that's OK, they'll be equivalent.
    let WindowParams{x, y, window, scale, ..} = query.clone();
    // Find the GCD between all of these:
    let gcd = [x, y, window, scale].into_iter().reduce(|a, b| a.gcd(&b)).unwrap_or_else(|| 1.into());
    let query = {
        let mut q = query;
        q.x /= &gcd;
        q.y /= &gcd;
        q.window /= &gcd;
        q.scale /= &gcd;
        q
    };

    // TODO: Don't pass the query string through, re-render it;
    // leads to incorrect caching of the default image

    html! {
        (DOCTYPE)
        head {
            title { "Fractal Farlands" }
            link rel="stylesheet" href="static/style.css";
            script src="static/app.js" async {}
        }
        body {
            (interface_body(uri.query().unwrap_or(""), &query))
        }
    }
}

fn interface_body(query_str: &str, query: &WindowParams) -> Markup {
    html! {
        form id="form-rerender" action="/" autocomplete="off" class="parameters" {
            h2 { "Target area" }
            p {
                label { "Center X (numerator):" }
                input id="input-x" name="x" type="number" value=(query.x);
                " "

                label { "Center Y (numerator):" }
                input id="input-y" name="y" type="number" value=(query.y);
                " "

                label { "Window size (numerator)" }
                input id="input-window" name="window" type="number" value=(query.window);
                " "

                label { "Scale (denominator):" }
                input id="input-scale" name="scale" type="number" value=(query.scale);
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
        p {
            a href="/" { "Reset" }
        }

        div {
            h2 { "Rendering"}
            p {
                "Click on an image to re-center. Zoom: "
                button id="button-out" { " - " }
                " "
                button id="button-in" { " + " }
            }
            p { (format!("Parameters: {:?}", query)) }


            @for format in ff_core::mandelbrot::formats() {
                (render(query_str, &query.fractal, format, query.res))
            }
        }

    }
}

fn render(query_str: &str, fractal: &str, numeric: &str, size: usize) -> Markup {
    html! {
        div class="render-pane" {
            h3 { (numeric) }
            img src=(format!("/render/{}/{}?{}", fractal, numeric, query_str)) width=(size) height=(size) class="img-fractal";
        }
    }
}
