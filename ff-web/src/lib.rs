/// HTTP serving for Fractal Farlands.
///
/// The library implements routing, state, etc. for an FF server;
/// the binary starts up a runtime and Warp server to handle requests.
///
/// All dynamic paths take query parameters:
/// - res: Integer width & height in pixels. (Rendering is always square.)
/// - iter: Maximum number of iterations.
///
/// - window: Numerator for window width/height. Defaults to 4.
/// - x: Numerator of X offset of upper-left corner. Defaults to -2.
/// - y: Numerator of Y offset of upper-left corner. Defaults to -2.
/// - scale: Denominator for x, y, and window. Defaults to 1.
///
/// Dynamic paths are:
/// - `/`: HTML interface view. View parameters are filled by query params.
/// - `/render/:fractal/:numeric`: Render the given fractal using the given numeric format, in the query-provided window.
///
/// Static paths are:
/// - `/static/...`: Serve the provided static content (JS, CSS)
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use num_bigint::BigInt;
use serde::de::{Deserialize, Deserializer};

mod interface;
mod render;
mod static_content;

pub fn root_routes(_web_rt: tokio::runtime::Handle) -> axum::Router {
    Router::new()
        .route(
            "/",
            get(interface::interface),
        )
        .route(
            "/render/:fractal/:numeric",
            get(
                |Path((fractal, numeric)): Path<(String, String)>,
                 Query(window_params): Query<WindowParams>| {
                    render::render(fractal, numeric, window_params)
                },
            ),
        )
        .route("/static/:file", get(static_content::get))
}

#[derive(serde::Deserialize, Debug)]
struct WindowParams {
    #[serde(default = "WindowParams::default_res")]
    res: usize,
    #[serde(default = "WindowParams::default_iters")]
    iters: usize,
    #[serde(default = "WindowParams::default_fractal")]
    fractal: String,

    #[serde(
        default = "WindowParams::default_window",
        deserialize_with = "parse_bigint"
    )]
    window: BigInt,
    #[serde(
        default = "WindowParams::default_coord",
        deserialize_with = "parse_bigint"
    )]
    x: BigInt,
    #[serde(
        default = "WindowParams::default_coord",
        deserialize_with = "parse_bigint"
    )]
    y: BigInt,
    #[serde(
        default = "WindowParams::default_scale",
        deserialize_with = "parse_bigint"
    )]
    scale: BigInt,
}

/// Converter to parse BigInt via string.
fn parse_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    buf.parse().map_err(serde::de::Error::custom)
}

impl WindowParams {
    fn default_fractal() -> String {
        "mandelbrot".to_owned()
    }
    fn default_res() -> usize {
        512
    }
    fn default_iters() -> usize {
        512
    }

    fn default_window() -> BigInt {
        4.into()
    }
    fn default_coord() -> BigInt {
        0.into()
    }
    fn default_scale() -> BigInt {
        1.into()
    }
}
