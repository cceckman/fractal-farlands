use std::sync::Arc;

use crate::WindowParams;
/// User-interface rendering for Fractal Farlands Newton Fractal.
use axum::{
    extract::{OriginalUri, Path, Query},
    routing::get,
    Router,
};

use ff_render::RenderServer;
use maud::{html, Markup, DOCTYPE};
use num::Integer;

pub fn router(render_server: RenderServer) -> Router {
    let srv = Arc::new(render_server);
    Router::new().route("/", get(interface)).route(
        "/render/:numeric",
        get(
            |Path(numeric), Query(window_params): Query<WindowParams>| async move {
                let request = window_params.to_request("newton", numeric)?;
                crate::render::render(&srv, request).await
            },
        ),
    )
}

/// Render the user interface.
async fn interface(uri: OriginalUri, Query(query): Query<WindowParams>) -> Markup {
    // Simplify WindowParams where we can- before outputting to the user.
    // This may mean our query parameters don't match; that's OK, they'll be equivalent.
    let WindowParams {
        x,
        y,
        window,
        scale,
        ..
    } = query.clone();
    // Find the GCD between all of these:
    let gcd = [x, y, window, scale]
        .into_iter()
        .reduce(|a, b| a.gcd(&b))
        .unwrap_or_else(|| 1.into());
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
            title { "Fractal Farlands - newton" }
            link rel="stylesheet" href="/static/style.css";
            script src="/static/app.js" async {}
        }
        body {
            (interface_body(uri.query().unwrap_or(""), &query))
        }
    }
}

fn interface_body(query_str: &str, query: &WindowParams) -> Markup {
    html! {
        form id="form-rerender" action="." autocomplete="off" class="parameters" {
            h1 { 
                a href="/" { "Fractal Farlands" }
                "- newton"
            }
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
            a href="." { "Reset" }
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


            @for format in ff_core::newton::formats() {
                (render(query_str, format, query.res))
            }
        }

    }
}

fn render(query_str: &str, numeric: &str, size: usize) -> Markup {
    html! {
        div class="render-pane" {
            h3 { (numeric) }
            img src=(format!("render/{}?{}", numeric, query_str)) width=(size) height=(size) class="img-fractal";
        }
    }
}
