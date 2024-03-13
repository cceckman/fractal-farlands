use crate::{UiParams, WindowParams};
/// User-interface rendering for Fractal Farlands.
use axum::extract::Query;

use maud::{html, Markup, DOCTYPE};
use num::Integer;

/// Render the user interface.
pub async fn interface(Query(query): Query<UiParams>) -> Result<Markup, String> {
    // Simplify WindowParams where we can- before outputting to the user.
    // This may mean our query parameters don't match; that's OK, they'll be equivalent.
    let WindowParams{x, y, window, scale, ..} = query.window.clone();
    // Find the GCD between all of these:
    let gcd = [x, y, window, scale].into_iter().reduce(|a, b| a.gcd(&b)).unwrap_or_else(|| 1.into());
    let new_window = {
        let mut q = query.window.clone();
        q.x /= &gcd;
        q.y /= &gcd;
        q.window /= &gcd;
        q.scale /= &gcd;
        q
    };
    let new_query = UiParams{window: new_window, ..query};

    // TODO: Don't pass the query string through, re-render it;
    // leads to incorrect caching of the default image

    Ok(html! {
        (DOCTYPE)
        head {
            title { "Fractal Farlands" }
            link rel="stylesheet" href="static/style.css";
            script src="static/app.js" async {}
        }
        body {
            (interface_body(&new_query)?)
        }
    })
}

fn interface_body(query: &UiParams) -> Result<Markup, String> {
    Ok(html! {
        form id="form-rerender" action="/" autocomplete="off" class="parameters" {
            h2 { "Target area" }
            p {
                label { "Center X (numerator):" }
                input id="input-x" name="x" type="number" value=(query.window.x);
                " "

                label { "Center Y (numerator):" }
                input id="input-y" name="y" type="number" value=(query.window.y);
                " "

                label { "Window size (numerator)" }
                input id="input-window" name="window" type="number" value=(query.window.window);
                " "

                label { "Scale (denominator):" }
                input id="input-scale" name="scale" type="number" value=(query.window.scale);
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
                input name="res" type="number" value=(query.window.res);
                " "

                label { "Max iterations:" }
                input name="iters" type="number" value=(query.window.iters);
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
                (render(&query.fractal, format, &query.window)?)
            }
        }
    })
}

fn render(fractal: &str, format: &str, window: &WindowParams) -> Result<Markup, String> {
    let urlencoded = serde_urlencoded::to_string(window).map_err(|err| format!("error encoding URL; parameters: {:?}, error: {}", window, err))?;
    Ok(html! {
        div class="render-pane" {
            h3 { (format) }
            img src=(format!("/render/{}/{}?{}", fractal, format, urlencoded)) width=(window.res) height=(window.res) class="img-fractal";
        }
    })
}
