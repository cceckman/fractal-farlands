//! HTTP serving for Fractal Farlands.
//!
//! The library implements routing, state, etc. for an FF server;
//! the binary starts up a runtime and Warp server to handle requests.
//!
//! All dynamic paths take query parameters:
//! - res: Integer width & height in pixels. (Rendering is always square.)
//! - iter: Maximum number of iterations.
//!
//! - window: Numerator for window width/height. Defaults to 4.
//! - x: Numerator of X offset of upper-left corner. Defaults to -2.
//! - y: Numerator of Y offset of upper-left corner. Defaults to -2.
//! - scale: Denominator for x, y, and window. Defaults to 1.
//!
//! Dynamic paths are:
//! - `/`: HTML interface view. View parameters are filled by query params.
//! - `/render/:fractal/:numeric`: Render the given fractal using the given numeric format, in the query-provided window.
//!
//! Static paths are:
//! - `/static/...`: Serve the provided static content (JS, CSS)
use axum::{routing::get, Router};
use ff_core::{CommonParams, FractalParams, RenderRequest, Size};
use num::BigRational;
use num_bigint::BigInt;
use serde::de::{Deserialize, Deserializer};

mod mandelbrot;
mod newton;
mod render;
mod static_content;

pub fn root_routes() -> Result<axum::Router, String> {
    tracing::info!("constructing router");
    Ok(Router::new()
        .route("/", get(static_content::get_index))
        .nest("/mandelbrot/", mandelbrot::router(ff_render::RenderServer::new()?))
        .nest("/newton/", newton::router(ff_render::RenderServer::new()?))
        .route("/static/:file", get(static_content::get)))
}

#[derive(serde::Deserialize, Debug, Clone)]
struct WindowParams {
    #[serde(default = "WindowParams::default_res")]
    res: usize,
    #[serde(default = "WindowParams::default_iters")]
    iters: usize,

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

impl WindowParams {
    fn to_request(self, fractal: &str, numeric: String) -> Result<RenderRequest, String> {
        // Web request uses center; internals use a window.
        // Compute the window.
        let half_range = self.window / 2;

        let range = |v: &BigInt| {
            let start = BigRational::new(v - &half_range, self.scale.clone());
            let end = BigRational::new(v + &half_range, self.scale.clone());
            start..end
        };

        let common = CommonParams {
            size: Size {
                width: self.res,
                height: self.res,
            },
            x: range(&self.x),
            y: range(&self.y),
            numeric,
        };
        let fractal = match fractal {
            "mandelbrot" => Ok(FractalParams::Mandelbrot { iters: self.iters }),
            "newton" => Ok(FractalParams::Newton { iters: self.iters }),
            v => Err(format!("unknown fractal '{}'", v)),
        }?;
        Ok(RenderRequest { common, fractal })
    }
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
    fn default_res() -> usize {
        512
    }
    fn default_iters() -> usize {
        16
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
