/// User-interface rendering for Fractal Farlands.

use axum::extract::Query;
use crate::WindowParams;

use maud::{html,Markup};

/// Render the user interface.
pub async fn interface(Query(query): Query<WindowParams>) -> Markup {
    html! {
        title { "Fractal Farlands" }
        link rel="stylesheet" href="static/style.css";
        body {
            (interface_body(&query))
        }
    }
}

fn interface_body(query: &WindowParams) -> Markup {
    html!{
        form action="/" autocomplete="off" {
            h2 { "Target area" }
            p {
                label { "X (numerator):" }
                input name="x" type="number" value=(query.x);
                " "

                label { "Y (numerator):" }
                input name="y" type="number" value=(query.y);
                " "

                label { "Window (numerator)" }
                input name="window" type="number" value=(query.window);
                " "

                label { "Scale (denominator):" }
                input name="scale" type="number" value=(query.scale);
                " "
            }
            h2 { "Rendering settings" }
            p {
                label { "Resolution (pixels):" }
                input name="res" type="number" value=(query.res);
                " "

                label { "Iterations:" }
                input name="iters" type="number" value=(query.iters);
                " "
                br;
            }

            input text="Go" type="submit";
        }

        div {
            h2 { "Rendering results "}
            p { (format!("Parameters: {:?}", query)) }

            (render("f32", query.res))
            (render("f64", query.res))
        }

    }
}

fn render(name: &'static str, size: usize) -> Markup {
    html!{
        div class="render-pane" {
            h3 { (name) }
            img src=(format!("/render/{}", name)) width=(size) height=(size);
        }
    }
}